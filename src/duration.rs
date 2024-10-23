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

#[test]
fn test_seconds_std_duration() {
	assert_eq!(
		serde_json::from_str::<Seconds<StdDuration>>("567").unwrap(),
		Seconds(StdDuration::from_secs(567))
	);
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

#[test]
fn test_ms_std_duration() {
	assert_eq!(
		serde_json::from_str::<Ms<StdDuration>>("567").unwrap(),
		Ms(StdDuration::from_millis(567))
	);
}
