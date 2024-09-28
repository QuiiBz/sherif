use anyhow::{anyhow, Result};
use semver::{Prerelease, Version, VersionReq};
use std::{cmp::Ordering, fmt::Display};

#[derive(Debug, PartialEq, Eq, Clone)]
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
        let mut ordering = self.patch().cmp(&other.patch());

        ordering = match self.minor().cmp(&other.minor()) {
            Ordering::Equal => ordering,
            new_ordering => new_ordering,
        };

        ordering = match self.major().cmp(&other.major()) {
            Ordering::Equal => ordering,
            new_ordering => new_ordering,
        };

        match self.prerelease().cmp(&other.prerelease()) {
            Ordering::Equal => ordering,
            new_ordering => new_ordering,
        }
    }

    pub fn is_valid(&self) -> bool {
        match self {
            Self::Exact(_) => true,
            Self::Range(version) => !version.comparators.is_empty(),
        }
    }
}
