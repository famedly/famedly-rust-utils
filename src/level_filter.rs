use serde::{de, Deserialize, Serialize};

/// LevelFilter wrapper with from trait implementations for `tracing`.
/// ```
/// # use famedly_rust_utils::LevelFilter;
/// use tracing::level_filters::LevelFilter as LF;
/// for (tlvl, s) in [
/// 	(LF::OFF, "off"),
/// 	(LF::ERROR, "error"),
/// 	(LF::WARN, "warn"),
/// 	(LF::INFO, "info"),
/// 	(LF::DEBUG, "debug"),
/// 	(LF::TRACE, "trace"),
/// ] {
/// 	let lvl: LevelFilter =
/// 		serde_json::from_value(serde_json::json!(s)).unwrap();
/// 	assert_eq!(tlvl, LF::from(lvl));
/// }
/// ```
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct LevelFilter(pub tracing::level_filters::LevelFilter);

impl std::fmt::Display for LevelFilter {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		self.0.fmt(f)
	}
}

impl Serialize for LevelFilter {
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		serializer.serialize_str(&self.to_string())
	}
}

impl<'de> Deserialize<'de> for LevelFilter {
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: de::Deserializer<'de>,
	{
		String::deserialize(deserializer)?.parse().map_err(de::Error::custom)
	}
}

impl std::str::FromStr for LevelFilter {
	type Err = tracing::level_filters::ParseLevelFilterError;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Ok(LevelFilter(tracing::level_filters::LevelFilter::from_str(s)?))
	}
}

impl From<LevelFilter> for tracing::level_filters::LevelFilter {
	fn from(level: LevelFilter) -> Self {
		level.0
	}
}

impl From<tracing::level_filters::LevelFilter> for LevelFilter {
	fn from(level: tracing::level_filters::LevelFilter) -> Self {
		LevelFilter(level)
	}
}

impl AsRef<tracing::level_filters::LevelFilter> for LevelFilter {
	fn as_ref(&self) -> &tracing::level_filters::LevelFilter {
		&self.0
	}
}

impl std::ops::Deref for LevelFilter {
	type Target = tracing::level_filters::LevelFilter;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

#[test]
fn test_serde() {
	use tracing::level_filters::LevelFilter as LF;
	for (tlvl, s) in [
		(LF::OFF, "off"),
		(LF::ERROR, "error"),
		(LF::WARN, "warn"),
		(LF::INFO, "info"),
		(LF::DEBUG, "debug"),
		(LF::TRACE, "trace"),
	] {
		let lvl: LevelFilter = serde_json::from_value(serde_json::json!(s)).unwrap();
		assert_eq!(tlvl, LF::from(lvl));

		let lvl: String = serde_json::to_string(&lvl).unwrap();
		assert_eq!(lvl, format!(r#""{}""#, s));
	}
}
