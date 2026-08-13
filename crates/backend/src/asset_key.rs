use std::fmt;
use std::num::NonZeroU32;
use std::str::FromStr;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct AssetKey {
    source: Uuid,
    copy: Option<NonZeroU32>,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum AssetKeyError {
    #[error("invalid asset id")]
    Uuid,
    #[error("invalid virtual copy index")]
    Index,
}

impl AssetKey {
    pub fn master(source: Uuid) -> Self {
        Self { source, copy: None }
    }

    pub fn copy(source: Uuid, index: NonZeroU32) -> Self {
        Self {
            source,
            copy: Some(index),
        }
    }

    pub fn source(&self) -> Uuid {
        self.source
    }

    pub fn is_copy(&self) -> bool {
        self.copy.is_some()
    }
}

impl fmt::Display for AssetKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.copy {
            Some(index) => write!(f, "{}_{}", self.source.as_hyphenated(), index),
            None => write!(f, "{}", self.source.as_hyphenated()),
        }
    }
}

impl FromStr for AssetKey {
    type Err = AssetKeyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some((source, index)) = s.split_once('_') else {
            let source = Uuid::parse_str(s).map_err(|_| AssetKeyError::Uuid)?;
            return Ok(Self::master(source));
        };
        let source = Uuid::parse_str(source).map_err(|_| AssetKeyError::Uuid)?;
        if index.starts_with('0') || !index.bytes().all(|b| b.is_ascii_digit()) {
            return Err(AssetKeyError::Index);
        }
        let index = index
            .parse::<NonZeroU32>()
            .map_err(|_| AssetKeyError::Index)?;
        Ok(Self::copy(source, index))
    }
}

impl From<Uuid> for AssetKey {
    fn from(source: Uuid) -> Self {
        Self::master(source)
    }
}

impl Serialize for AssetKey {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for AssetKey {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_str(AssetKeyVisitor)
    }
}

struct AssetKeyVisitor;

impl Visitor<'_> for AssetKeyVisitor {
    type Value = AssetKey;

    fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("an asset id, optionally suffixed with a virtual copy index")
    }

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
        AssetKey::from_str(v).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const UUID: &str = "6f2a1e4c-8b31-4f2a-9c7d-1a2b3c4d5e6f";

    #[test]
    fn round_trips() {
        for input in [UUID, &format!("{UUID}_1"), &format!("{UUID}_42")] {
            let key: AssetKey = input.parse().unwrap();
            assert_eq!(key.to_string(), input);
        }
    }

    #[test]
    fn normalizes_uppercase_source() {
        let key: AssetKey = format!("{}_2", UUID.to_uppercase()).parse().unwrap();
        assert_eq!(key.to_string(), format!("{UUID}_2"));
    }

    #[test]
    fn master_and_copy_differ() {
        let master: AssetKey = UUID.parse().unwrap();
        let copy: AssetKey = format!("{UUID}_1").parse().unwrap();
        assert!(!master.is_copy());
        assert!(copy.is_copy());
        assert_eq!(master.source(), copy.source());
        assert_ne!(master, copy);
    }

    #[test]
    fn rejects_bad_input() {
        for input in [
            format!("{UUID}_0"),
            format!("{UUID}_01"),
            format!("{UUID}_x"),
            format!("{UUID}_"),
            format!("{UUID}_1_2"),
            format!("{UUID}_-1"),
            format!("{UUID}_4294967296"),
            "not-a-uuid".into(),
            String::new(),
        ] {
            assert!(input.parse::<AssetKey>().is_err(), "accepted {input}");
        }
    }

    #[test]
    fn serde_uses_the_string_form() {
        let key: AssetKey = format!("{UUID}_3").parse().unwrap();
        let json = serde_json::to_string(&key).unwrap();
        assert_eq!(json, format!("\"{UUID}_3\""));
        assert_eq!(serde_json::from_str::<AssetKey>(&json).unwrap(), key);
    }
}
