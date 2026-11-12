use anyhow::{anyhow, Result};
use semver::{Prerelease, Version, VersionReq};
use std::{cmp::Ordering, fmt::Display};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum SemVersion {
    Exact(Version),
    Range(VersionReq),
}

impl Display for SemVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Exact(version) => f.write_str(&version.to_string()),
            Self::Range(version) => f.write_str(&version.to_string()),
        }
    }
}

impl SemVersion {
    pub fn parse(version: &str) -> Result<Self> {
        if let Ok(version) = Version::parse(version) {
            return Ok(Self::Exact(version));
        }

        if let Ok(version) = VersionReq::parse(version) {
            return Ok(Self::Range(version));
        }

        Err(anyhow!("Invalid version: {}", version))
    }

    pub fn patch(&self) -> u64 {
        match self {
            Self::Exact(version) => version.patch,
            Self::Range(version) => version
                .comparators
                .first()
                .map_or(0, |comparator| comparator.patch.unwrap_or(0)),
        }
    }

    pub fn minor(&self) -> u64 {
        match self {
            Self::Exact(version) => version.minor,
            Self::Range(version) => version
                .comparators
                .first()
                .map_or(0, |comparator| comparator.minor.unwrap_or(0)),
        }
    }

    pub fn major(&self) -> u64 {
        match self {
            Self::Exact(version) => version.major,
            Self::Range(version) => version
                .comparators
                .first()
                .map_or(0, |comparator| comparator.major),
        }
    }

    pub fn prerelease(&self) -> Prerelease {
        match self {
            Self::Exact(version) => version.pre.clone(),
            Self::Range(version) => version
                .comparators
                .first()
                .map_or(Prerelease::EMPTY, |comparator| comparator.pre.clone()),
        }
    }

    pub fn cmp(&self, other: &Self) -> Ordering {
        // SemVer precedence: the prerelease only breaks ties between
        // equal major.minor.patch versions (and a stable version sorts
        // above its own prereleases, which `Prerelease`'s `Ord` handles).
        self.major()
            .cmp(&other.major())
            .then_with(|| self.minor().cmp(&other.minor()))
            .then_with(|| self.patch().cmp(&other.patch()))
            .then_with(|| self.prerelease().cmp(&other.prerelease()))
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Self::Exact(_) => true,
            Self::Range(version) => !version.comparators.is_empty(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn cmp(a: &str, b: &str) -> Ordering {
        SemVersion::parse(a)
            .unwrap()
            .cmp(&SemVersion::parse(b).unwrap())
    }

    #[test]
    fn cmp_stable() {
        assert_eq!(cmp("1.2.3", "1.2.3"), Ordering::Equal);
        assert_eq!(cmp("1.2.4", "1.2.3"), Ordering::Greater);
        assert_eq!(cmp("1.3.0", "1.2.9"), Ordering::Greater);
        assert_eq!(cmp("2.0.0", "1.9.9"), Ordering::Greater);
        assert_eq!(cmp("^1.10.0", "~1.9.0"), Ordering::Greater);
    }

    #[test]
    fn cmp_prerelease_same_version() {
        assert_eq!(cmp("5.0.0", "5.0.0-rc.1"), Ordering::Greater);
        assert_eq!(cmp("5.0.0-next.4", "5.0.0-next.3"), Ordering::Greater);
        assert_eq!(cmp("5.0.0-beta.1", "5.0.0-alpha.9"), Ordering::Greater);
    }

    #[test]
    fn cmp_prerelease_other_version() {
        assert_eq!(cmp("15.0.0-rc.0", "14.2.3"), Ordering::Greater);
        assert_eq!(cmp("2.0.0-beta.1", "1.4.0"), Ordering::Greater);
        assert_eq!(cmp("1.10.0-rc.1", "^1.9.0"), Ordering::Greater);
        assert_eq!(cmp("1.2.4-rc.1", "1.2.3"), Ordering::Greater);
        assert_eq!(cmp("1.4.0", "2.0.0-beta.1"), Ordering::Less);
        assert_eq!(cmp("2.0.0-beta.1", "1.9.0-rc.1"), Ordering::Greater);
    }
}
