use std::{fmt, str::FromStr};

use anyhow::bail;
use semver::{BuildMetadata, Prerelease, Version};
use serde::{
    Deserialize, Deserializer,
    de::{Error as SerdeError, Visitor},
};

#[derive(Debug)]
pub enum GodotVersionParseError {
    Empty,
    Invalid(String),
}

impl std::error::Error for GodotVersionParseError {}

impl fmt::Display for GodotVersionParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => {
                f.write_str("Empty version string")
            }
            Self::Invalid(s) => {
                write!(f, "Invalid version \"{}\"", s)
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct GodotVersion {
    pub version: Version,
}

impl GodotVersion {
    // TODO: Consider implementing this as FromStr instead.
    pub fn parse(v: &str) -> anyhow::Result<Self> {
        let Some((version, suffix)) = v.split_once('-') else {
            // TODO: Add error context.
            bail!("Invalid Godot version {}", v);
        };

        let mut split_version = version.split('.');
        let Some(major) = split_version.next().and_then(|m| m.parse().ok()) else {
            // TODO: Add error context.
            bail!("Invalid Godot version {}", v);
        };
        let minor = if let Some(minor) = split_version.next() {
            // TODO: Add error context.
            minor.parse()?
        } else {
            0
        };
        let patch = if let Some(patch) = split_version.next() {
            // TODO: Add error context.
            patch.parse()?
        } else {
            0
        };

        let pre = match suffix {
            "stable" | "" => Prerelease::EMPTY,
            // TODO: Add error context.
            _ => Prerelease::new(suffix)?,
        };

        let version = Version {
            major,
            minor,
            patch,
            pre,
            build: BuildMetadata::EMPTY,
        };

        Ok(GodotVersion {
            version,
        })
    }

    pub fn is_stable(&self) -> bool {
        self.version.pre.is_empty()
    }

    pub fn to_full_version(&self) -> String {
        let mut full_version = self.version.to_string();
        if self.is_stable() {
            full_version.push_str("{}-stable");
        }
        full_version
    }
}

impl FromStr for GodotVersion {
    type Err = GodotVersionParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Err(GodotVersionParseError::Empty)
    }
}

impl fmt::Display for GodotVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.version)
    }
}

impl<'de> Deserialize<'de> for GodotVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        // TODO: Implement. Deserialize to string and build a GodotVersion out of it.
        struct GodotVersionVisitor;

        impl<'de> Visitor<'de> for GodotVersionVisitor {
            type Value = GodotVersion;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("Godot version")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: SerdeError,
            {
                GodotVersion::parse(v).map_err(SerdeError::custom)
            }
        }

        deserializer.deserialize_str(GodotVersionVisitor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_empty_fails() {
        let result = GodotVersion::parse("");
        assert!(result.is_err());
    }
}
