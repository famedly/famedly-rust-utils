//! # Duration wrapper for serde
//!
//! - `Ms<std::time::Duration>` - deserializes `u64` as milliseconds
//! - `Seconds<std::time::Duration>` - deserializes `u64` as seconds
//! - `Ms<time::Duration>` - deserializes `i64` into as milliseconds
//! - `Seconds<time::Duration>` - deserializes `i64` into as seconds
//!
//! ```
//! use famedly_rust_utils::duration::Ms;
//! assert_eq!(
//! 	*serde_json::from_str::<Ms<std::time::Duration>>("567").unwrap(),
//! 	std::time::Duration::from_millis(567)
//! );
//! ```
use std::time::Duration as StdDuration;

#[cfg(feature = "time")]
use time::Duration as TimeDuration;

crate::define_generic_wrapper! {
	"Helper wrapper to use in configs to deserialize durations from seconds",
	Seconds:

	{
		StdDuration,
		|deserializer| u64::deserialize(deserializer).map(StdDuration::from_secs),
		|serializer: S, x: StdDuration| serializer.serialize_u64(x.as_secs())
	},

	feature "time";
	{
		TimeDuration,
		|deserializer| i64::deserialize(deserializer).map(TimeDuration::seconds),
		|serializer: S, x: TimeDuration| serializer.serialize_i64(x.whole_seconds())
	}
}

impl Seconds<StdDuration> {
	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_uint(s: u64) -> Self {
		Seconds(StdDuration::from_secs(s))
	}
}

#[cfg(feature = "time")]
impl Seconds<TimeDuration> {
	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_int(s: i64) -> Self {
		Seconds(TimeDuration::seconds(s))
	}
}

#[test]
fn test_seconds_std_duration() {
	assert_eq!(
		serde_json::from_str::<Seconds<StdDuration>>("567").unwrap(),
		Seconds::from_uint(567)
	);
}

crate::define_generic_wrapper! {
	"Helper wrapper to use in configs to deserialize durations from minutes",
	Minutes:

	{
		StdDuration,
		|deserializer| u64::deserialize(deserializer).map(|x| StdDuration::from_secs(x * 60)),
		|serializer: S, x: StdDuration| serializer.serialize_u64(x.as_secs() / 60)
	},

	feature "time";
	{
		TimeDuration,
		|deserializer| i64::deserialize(deserializer).map(TimeDuration::minutes),
		|serializer: S, x: TimeDuration| serializer.serialize_i64(x.whole_minutes())
	}
}

impl Minutes<StdDuration> {
	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_uint(m: u64) -> Self {
		Minutes(StdDuration::from_secs(m * 60))
	}
}

#[cfg(feature = "time")]
impl Minutes<TimeDuration> {
	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_int(m: i64) -> Self {
		Minutes(TimeDuration::minutes(m))
	}
}

#[test]
fn test_minutes_std_duration() {
	assert_eq!(
		serde_json::from_str::<Minutes<StdDuration>>("567").unwrap(),
		Minutes::from_uint(567)
	);
}

crate::define_generic_wrapper! {
	"Helper wrapper to use in configs to deserialize durations from hours",
	Hours:

	{
		StdDuration,
		|deserializer| u64::deserialize(deserializer).map(|x| StdDuration::from_secs(x * 60 * 60)),
		|serializer: S, x: StdDuration| serializer.serialize_u64(x.as_secs() / 60 / 60)
	},

	feature "time";
	{
		TimeDuration,
		|deserializer| i64::deserialize(deserializer).map(TimeDuration::hours),
		|serializer: S, x: TimeDuration| serializer.serialize_i64(x.whole_hours())
	}
}

impl Hours<StdDuration> {
	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_uint(h: u64) -> Self {
		Hours(StdDuration::from_secs(h * 60 * 60))
	}
}

#[cfg(feature = "time")]
impl Hours<TimeDuration> {
	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_int(h: i64) -> Self {
		Hours(TimeDuration::hours(h))
	}
}

#[test]
fn test_hours_std_duration() {
	assert_eq!(serde_json::from_str::<Hours<StdDuration>>("567").unwrap(), Hours::from_uint(567));
}

crate::define_generic_wrapper! {
	"Helper wrapper to use in configs to deserialize durations from milliseconds",
	Ms:

	{
		StdDuration,
		|deserializer| u64::deserialize(deserializer).map(StdDuration::from_millis),
		// |serializer: S, x: StdDuration| serializer.serialize_u128(x.as_millis())
	},

	feature "time";
	{
		TimeDuration,
		|deserializer| i64::deserialize(deserializer).map(TimeDuration::milliseconds),
		// |serializer: S, x: TimeDuration| serializer.serialize_i64(x.whole_milliseconds())
	}
}

impl Ms<StdDuration> {
	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_uint(ms: u64) -> Self {
		Ms(StdDuration::from_millis(ms))
	}
}

#[cfg(feature = "time")]
impl Ms<TimeDuration> {
	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_int(ms: i64) -> Self {
		Ms(TimeDuration::milliseconds(ms))
	}
}

#[test]
fn test_ms_std_duration() {
	assert_eq!(serde_json::from_str::<Ms<StdDuration>>("567").unwrap(), Ms::from_uint(567));
}
