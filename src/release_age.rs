//! Detection of minimum release age "bypasses" (exclusion lists) that are no
//! longer necessary.
//!
//! Package managers can gate installs behind a minimum release age (a cooldown
//! after a version is published), with an escape hatch listing packages that
//! should bypass the gate:
//!
//! | Package manager | File                  | Age                    | Bypass list                 |
//! | --------------- | --------------------- | ---------------------- | --------------------------- |
//! | pnpm            | `pnpm-workspace.yaml` | `minimumReleaseAge` (m)| `minimumReleaseAgeExclude`  |
//! | Yarn            | `.yarnrc.yml`         | `npmMinimalAgeGate` (m)| `npmPreapprovedPackages`    |
//! | Bun             | `bunfig.toml`         | `minimumReleaseAge` (s)| `minimumReleaseAgeExcludes` |
//!
//! Those bypasses are usually added to unblock an urgent upgrade, and are then
//! forgotten: once every version of the package is older than the gate, the
//! entry doesn't change anything anymore and can be removed.
//!
//! This is the only part of Sherif that needs network access. Publish dates
//! aren't available on disk: lockfiles only store resolved versions and
//! integrity hashes, and the metadata caches of the package managers strip the
//! `time` field of the registry documents. The abbreviated registry metadata
//! (`application/vnd.npm.install-v1+json`) doesn't contain it either, so we
//! fetch the full document - which is what package managers do to enforce the
//! minimum release age.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_REGISTRY: &str = "https://registry.npmjs.org";
const NPMRC: &str = ".npmrc";
/// Maximum number of registry requests done in parallel.
const MAX_CONCURRENCY: usize = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigSource {
    PnpmWorkspace,
    Yarnrc,
    Bunfig,
}

impl ConfigSource {
    pub fn file_name(&self) -> &'static str {
        match self {
            ConfigSource::PnpmWorkspace => "pnpm-workspace.yaml",
            ConfigSource::Yarnrc => ".yarnrc.yml",
            ConfigSource::Bunfig => "bunfig.toml",
        }
    }

    pub fn exclude_field(&self) -> &'static str {
        match self {
            ConfigSource::PnpmWorkspace => "minimumReleaseAgeExclude",
            ConfigSource::Yarnrc => "npmPreapprovedPackages",
            ConfigSource::Bunfig => "minimumReleaseAgeExcludes",
        }
    }

    pub fn age_field(&self) -> &'static str {
        match self {
            ConfigSource::PnpmWorkspace | ConfigSource::Bunfig => "minimumReleaseAge",
            ConfigSource::Yarnrc => "npmMinimalAgeGate",
        }
    }
}

/// An entry of a bypass list, e.g. `nx`, `@myorg/*` or `webpack@4.47.0 || 5.102.1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExcludeEntry {
    /// The entry as written in the config file.
    pub raw: String,
    /// The package name, without the version locator.
    pub name: String,
    /// The exact versions listed in the locator, if any.
    pub versions: Vec<String>,
}

impl ExcludeEntry {
    pub fn parse(raw: &str) -> Self {
        let raw = raw.trim().to_string();
        // Only split on a `@` that isn't the leading one of a scoped package.
        let (name, locator) = match raw[1..].find('@') {
            Some(index) => {
                let index = index + 1;
                (raw[..index].to_string(), Some(raw[index + 1..].to_string()))
            }
            None => (raw.clone(), None),
        };

        let versions = match locator {
            Some(locator) => locator
                // Yarn locators are prefixed with the protocol, e.g. `nx@npm:21.6.5`.
                .trim_start_matches("npm:")
                .split("||")
                .map(|version| version.trim().to_string())
                .collect(),
            None => Vec::new(),
        };

        // A locator that isn't a list of exact versions (e.g. `^21.0.0`) can match
        // versions that aren't published yet, so we can't tell anything about it.
        let versions = match versions
            .iter()
            .all(|version| semver::Version::parse(version).is_ok())
        {
            true => versions,
            false => Vec::new(),
        };

        Self {
            raw,
            name,
            versions,
        }
    }

    /// Patterns can't be resolved to a single package, and the gate also applies
    /// to transitive dependencies we don't know about, so we skip them.
    pub fn is_pattern(&self) -> bool {
        self.name.contains('*')
    }
}

#[derive(Debug)]
pub struct ReleaseAgeConfig {
    pub source: ConfigSource,
    /// The minimum release age, in seconds.
    pub max_age: u64,
    pub excludes: Vec<ExcludeEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PnpmWorkspaceReleaseAge {
    minimum_release_age: Option<u64>,
    #[serde(default)]
    minimum_release_age_exclude: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct YarnrcReleaseAge {
    npm_minimal_age_gate: Option<u64>,
    #[serde(default)]
    npm_preapproved_packages: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct Bunfig {
    install: Option<BunfigInstall>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BunfigInstall {
    minimum_release_age: Option<u64>,
    #[serde(default)]
    minimum_release_age_excludes: Vec<String>,
}

/// Collects every minimum release age configuration of the monorepo root. Files
/// that don't exist, can't be read or don't define both an age and a non-empty
/// bypass list are ignored.
pub fn collect_configs(root: &Path) -> Vec<ReleaseAgeConfig> {
    let mut configs = Vec::new();

    if let Some((age, excludes)) = read_config(root, ConfigSource::PnpmWorkspace, |content| {
        let workspace: PnpmWorkspaceReleaseAge = serde_yaml::from_str(content).ok()?;

        Some((
            // pnpm expresses the age in minutes.
            workspace.minimum_release_age? * 60,
            workspace.minimum_release_age_exclude,
        ))
    }) {
        configs.push(ReleaseAgeConfig {
            source: ConfigSource::PnpmWorkspace,
            max_age: age,
            excludes,
        });
    }

    if let Some((age, excludes)) = read_config(root, ConfigSource::Yarnrc, |content| {
        let yarnrc: YarnrcReleaseAge = serde_yaml::from_str(content).ok()?;

        Some((
            // Yarn expresses the age in minutes.
            yarnrc.npm_minimal_age_gate? * 60,
            yarnrc.npm_preapproved_packages,
        ))
    }) {
        configs.push(ReleaseAgeConfig {
            source: ConfigSource::Yarnrc,
            max_age: age,
            excludes,
        });
    }

    if let Some((age, excludes)) = read_config(root, ConfigSource::Bunfig, |content| {
        let bunfig: Bunfig = toml::from_str(content).ok()?;
        let install = bunfig.install?;

        Some((
            // Bun expresses the age in seconds.
            install.minimum_release_age?,
            install.minimum_release_age_excludes,
        ))
    }) {
        configs.push(ReleaseAgeConfig {
            source: ConfigSource::Bunfig,
            max_age: age,
            excludes,
        });
    }

    configs
}

fn read_config<F>(root: &Path, source: ConfigSource, parse: F) -> Option<(u64, Vec<ExcludeEntry>)>
where
    F: Fn(&str) -> Option<(u64, Vec<String>)>,
{
    let content = fs::read_to_string(root.join(source.file_name())).ok()?;
    let (age, excludes) = parse(&content)?;

    if age == 0 || excludes.is_empty() {
        return None;
    }

    Some((
        age,
        excludes
            .iter()
            .map(|exclude| ExcludeEntry::parse(exclude))
            .collect(),
    ))
}

/// How many times the minimum release age a package must have been silent for
/// an exclusion without a version to be reported. A package that publishes
/// often will have a new version inside the gate again tomorrow, so only
/// packages that look dormant are worth reporting.
const DORMANT_FACTOR: u64 = 10;

#[derive(Debug, PartialEq, Eq)]
pub enum Verdict {
    /// Every version the entry can unblock is past the gate, and always will
    /// be: the entry pins versions that are already published.
    NoEffect { published_at: u64 },
    /// The entry doesn't pin any version, but the package hasn't published
    /// anything in a long time: the exclusion isn't unblocking anything.
    Dormant { published_at: u64 },
    /// The entry still bypasses the gate for at least one version.
    Active,
    /// Not enough information to decide.
    Unknown,
}

/// Publish timestamps of a package, in seconds since the Unix epoch.
pub type PublishTimes = BTreeMap<String, u64>;

/// Decides whether an entry still has an effect, given the publish times of the
/// package. Pure: the registry lookup is done by the caller.
pub fn evaluate(entry: &ExcludeEntry, times: &PublishTimes, now: u64, max_age: u64) -> Verdict {
    // No version: the entry also bypasses the gate for versions that aren't
    // published yet, so we can only tell that it currently unblocks nothing.
    if entry.versions.is_empty() {
        let Some(published) = times.values().copied().max() else {
            return Verdict::Unknown;
        };

        return match now.saturating_sub(published) >= max_age * DORMANT_FACTOR {
            true => Verdict::Dormant {
                published_at: published,
            },
            false => Verdict::Active,
        };
    }

    // Pinned versions: they are already published, so once they are past the
    // gate they stay past it.
    let published = entry
        .versions
        .iter()
        .filter_map(|version| times.get(version).copied())
        .collect::<Vec<_>>();

    // At least one version is unknown to the registry.
    if published.len() != entry.versions.len() {
        return Verdict::Unknown;
    }

    let Some(published) = published.into_iter().max() else {
        return Verdict::Unknown;
    };

    match now.saturating_sub(published) >= max_age {
        true => Verdict::NoEffect {
            published_at: published,
        },
        false => Verdict::Active,
    }
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

#[derive(Debug, Deserialize)]
struct Packument {
    #[serde(default)]
    time: BTreeMap<String, String>,
}

/// The public npm registry, and the scopes that use another one. Packages of
/// those scopes are never queried: we only talk to the public registry, to
/// avoid sending requests - and package names - to private registries.
#[derive(Debug, PartialEq, Eq)]
pub struct Registry {
    pub url: String,
    pub custom_scopes: Vec<String>,
}

impl Registry {
    pub fn handles(&self, name: &str) -> bool {
        match name.split_once('/') {
            Some((scope, _)) => !self.custom_scopes.iter().any(|custom| custom == scope),
            None => true,
        }
    }
}

/// The registry to use, or `None` if the monorepo uses a custom one for every
/// package.
pub fn registry(root: &Path) -> Option<Registry> {
    let default = Registry {
        url: DEFAULT_REGISTRY.to_string(),
        custom_scopes: Vec::new(),
    };

    let npmrc = match fs::read_to_string(root.join(NPMRC)) {
        Ok(npmrc) => npmrc,
        Err(_) => return Some(default),
    };

    let mut custom_scopes = Vec::new();

    for line in npmrc.lines() {
        let line = line.trim();

        if line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        let Some((key, value)) = line.split_once('=') else {
            continue;
        };

        let key = key.trim();
        let value = value.trim().trim_end_matches('/');

        if key == "registry" {
            if value != DEFAULT_REGISTRY {
                return None;
            }
        } else if let Some(scope) = key.strip_suffix(":registry") {
            if value != DEFAULT_REGISTRY {
                custom_scopes.push(scope.to_string());
            }
        }
    }

    Some(Registry {
        url: DEFAULT_REGISTRY.to_string(),
        custom_scopes,
    })
}

/// Fetches the publish times of the given packages. Any package that can't be
/// fetched is missing from the result: the rule is best-effort, and must never
/// fail because the registry is unreachable.
pub fn fetch_publish_times(registry: &str, names: &[String]) -> BTreeMap<String, PublishTimes> {
    let mut result = BTreeMap::new();

    for chunk in names.chunks(MAX_CONCURRENCY) {
        let fetched = std::thread::scope(|scope| {
            let handles = chunk
                .iter()
                .map(|name| scope.spawn(move || (name.clone(), fetch_publish_time(registry, name))))
                .collect::<Vec<_>>();

            handles
                .into_iter()
                .filter_map(|handle| handle.join().ok())
                .collect::<Vec<_>>()
        });

        for (name, times) in fetched {
            if let Some(times) = times {
                result.insert(name, times);
            }
        }
    }

    result
}

fn fetch_publish_time(registry: &str, name: &str) -> Option<PublishTimes> {
    let agent = ureq::Agent::config_builder()
        .timeout_connect(Some(std::time::Duration::from_secs(5)))
        .timeout_global(Some(std::time::Duration::from_secs(30)))
        .build()
        .new_agent();

    // The abbreviated metadata doesn't contain publish times, so we need the
    // full document.
    let mut response = agent
        .get(format!("{}/{}", registry, name.replace('/', "%2f")))
        .header("Accept", "application/json")
        .call()
        .ok()?;

    // Packuments of popular packages can be tens of megabytes.
    let body = response
        .body_mut()
        .with_config()
        .limit(256 * 1024 * 1024)
        .read_to_string()
        .ok()?;
    let packument: Packument = serde_json::from_str(&body).ok()?;

    Some(
        packument
            .time
            .into_iter()
            .filter(|(version, _)| version != "created" && version != "modified")
            .filter_map(|(version, date)| Some((version, parse_date(&date)?)))
            .collect(),
    )
}

/// Parses an ISO 8601 date as returned by the registry, e.g.
/// `2026-07-30T22:40:12.794Z`, into seconds since the Unix epoch.
fn parse_date(date: &str) -> Option<u64> {
    let (date, time) = date.split_once('T')?;
    let mut date = date.split('-');
    let year: i64 = date.next()?.parse().ok()?;
    let month: i64 = date.next()?.parse().ok()?;
    let day: i64 = date.next()?.parse().ok()?;

    let mut time = time.trim_end_matches('Z').split(':');
    let hours: u64 = time.next()?.parse().ok()?;
    let minutes: u64 = time.next()?.parse().ok()?;
    let seconds: u64 = time.next()?.split('.').next()?.parse().ok()?;

    Some(days_from_civil(year, month, day)? * 86400 + hours * 3600 + minutes * 60 + seconds)
}

/// Days since the Unix epoch, using Howard Hinnant's `days_from_civil` algorithm.
fn days_from_civil(year: i64, month: i64, day: i64) -> Option<u64> {
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    let year = year - i64::from(month <= 2);
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    u64::try_from(era * 146097 + day_of_era - 719468).ok()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_entries() {
        let entry = ExcludeEntry::parse("nx");
        assert_eq!(entry.name, "nx");
        assert!(entry.versions.is_empty());
        assert!(!entry.is_pattern());

        let entry = ExcludeEntry::parse("@myorg/*");
        assert_eq!(entry.name, "@myorg/*");
        assert!(entry.is_pattern());

        let entry = ExcludeEntry::parse("@types/node@22.0.0");
        assert_eq!(entry.name, "@types/node");
        assert_eq!(entry.versions, vec!["22.0.0"]);

        let entry = ExcludeEntry::parse("webpack@4.47.0 || 5.102.1");
        assert_eq!(entry.name, "webpack");
        assert_eq!(entry.versions, vec!["4.47.0", "5.102.1"]);

        // Yarn locators.
        let entry = ExcludeEntry::parse("nx@npm:21.6.5");
        assert_eq!(entry.name, "nx");
        assert_eq!(entry.versions, vec!["21.6.5"]);

        // Ranges can match versions that aren't published yet.
        let entry = ExcludeEntry::parse("nx@^21.0.0");
        assert_eq!(entry.name, "nx");
        assert!(entry.versions.is_empty());
    }

    #[test]
    fn parse_dates() {
        assert_eq!(parse_date("1970-01-01T00:00:00.000Z"), Some(0));
        assert_eq!(parse_date("2026-07-30T22:40:12.794Z"), Some(1785451212));
        assert_eq!(parse_date("2024-02-29T12:00:00Z"), Some(1709208000));
        assert_eq!(parse_date("not-a-date"), None);
    }

    #[test]
    fn evaluate_entries() {
        let now = 10_000_000;
        let max_age = 86400;
        let times = PublishTimes::from([
            ("1.0.0".to_string(), 100),
            ("2.0.0".to_string(), now - 3600),
        ]);

        // The package published something recently.
        let entry = ExcludeEntry::parse("nx");
        assert_eq!(evaluate(&entry, &times, now, max_age), Verdict::Active);

        // ...it also publishes often enough that it will keep bypassing the
        // gate, even though the last publish is past it.
        let recent = PublishTimes::from([("1.0.0".to_string(), now - max_age * 2)]);
        assert_eq!(evaluate(&entry, &recent, now, max_age), Verdict::Active);

        // ...but this one has been silent for a long time.
        let dormant = PublishTimes::from([("1.0.0".to_string(), now - max_age * 20)]);
        assert_eq!(
            evaluate(&entry, &dormant, now, max_age),
            Verdict::Dormant {
                published_at: now - max_age * 20
            }
        );

        // A pinned version that is past the gate always will be.
        let entry = ExcludeEntry::parse("nx@1.0.0");
        assert_eq!(
            evaluate(&entry, &times, now, max_age),
            Verdict::NoEffect { published_at: 100 }
        );

        // One of the pinned versions is still gated.
        let entry = ExcludeEntry::parse("nx@1.0.0 || 2.0.0");
        assert_eq!(evaluate(&entry, &times, now, max_age), Verdict::Active);

        // Unknown version.
        let entry = ExcludeEntry::parse("nx@3.0.0");
        assert_eq!(evaluate(&entry, &times, now, max_age), Verdict::Unknown);

        // No version at all.
        let entry = ExcludeEntry::parse("nx");
        assert_eq!(
            evaluate(&entry, &PublishTimes::new(), now, max_age),
            Verdict::Unknown
        );
    }

    #[test]
    fn collect_pnpm_config() {
        let configs = collect_configs(Path::new("fixtures/release-age-pnpm"));

        assert_eq!(configs.len(), 1);
        assert_eq!(configs[0].source, ConfigSource::PnpmWorkspace);
        assert_eq!(configs[0].max_age, 1440 * 60);
        assert_eq!(
            configs[0]
                .excludes
                .iter()
                .map(|exclude| exclude.raw.as_str())
                .collect::<Vec<_>>(),
            vec!["nx@21.6.5", "@myorg/*"]
        );
    }

    #[test]
    fn collect_no_config() {
        // No release age configured at all.
        assert!(collect_configs(Path::new("fixtures/pnpm")).is_empty());
        // A release age, but no bypass to check.
        assert!(collect_configs(Path::new("fixtures/release-age-no-exclude")).is_empty());
    }

    #[test]
    fn collect_bun_and_yarn_configs() {
        let configs = collect_configs(Path::new("fixtures/release-age-bun-yarn"));

        assert_eq!(configs.len(), 2);
        assert_eq!(configs[0].source, ConfigSource::Yarnrc);
        assert_eq!(configs[0].max_age, 1440 * 60);
        assert_eq!(configs[0].excludes[0].name, "@myorg/tools");
        assert_eq!(configs[1].source, ConfigSource::Bunfig);
        assert_eq!(configs[1].max_age, 86400);
        assert_eq!(configs[1].excludes[0].name, "typescript");
    }

    #[test]
    fn custom_registry_disables_the_rule() {
        let default = registry(Path::new("fixtures/pnpm")).unwrap();

        assert_eq!(default.url, DEFAULT_REGISTRY);
        assert!(default.handles("nx"));
        assert!(default.handles("@myorg/tools"));

        // A custom registry for every package.
        assert!(registry(Path::new("fixtures/release-age-custom-registry")).is_none());

        // A custom registry for a single scope.
        let scoped = registry(Path::new("fixtures/release-age-custom-scope")).unwrap();

        assert_eq!(scoped.custom_scopes, vec!["@myorg"]);
        assert!(scoped.handles("nx"));
        assert!(!scoped.handles("@myorg/tools"));
    }
}
