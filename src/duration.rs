// SPDX-FileCopyrightText: 2025 Famedly GmbH (info@famedly.com)
//
// SPDX-License-Identifier: Apache-2.0

//! # `Duration` wrapper for [`serde`]
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

#[cfg(feature = "schemars")]
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize, Serializer};

#[doc(hidden)]
macro_rules! define_generic_wrapper {
	($doc:expr, $name:ident: $( $(feature $feat:expr; )? { $t:ty, $deser:expr, $ser:expr }),*) => {
		#[doc = $doc]
		#[derive(Debug, PartialEq, Eq, Clone, Default)]
		#[repr(transparent)]
		pub struct $name<D>(pub D);

		impl<D> $name<D> {
			#[allow(missing_docs)]
			pub fn into_inner(self) -> D {
				self.0
			}
		}

		impl<D> From<D> for $name<D> {
			fn from(duration: D) -> Self {
				$name(duration)
			}
		}

		impl<D> std::ops::Deref for $name<D> {
			type Target = D;
			fn deref(&self) -> &Self::Target {
				&self.0
			}
		}

		impl<D> AsRef<D> for $name<D> {
			fn as_ref(&self) -> &D {
				&self.0
			}
		}

		impl<D: std::fmt::Display> std::fmt::Display for $name<D> {
			fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
				self.0.fmt(f)
			}
		}

		$(
			#[cfg(feature = "schemars")]
			impl JsonSchema for $name<$t> {
				fn schema_name() -> std::borrow::Cow<'static, str> {
					concat!("DurationIn", stringify!($name)).into()
				}
				fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
					schemars::json_schema!({"type": "integer"})
				}
				fn inline_schema() -> bool {
					true
				}
			}

			$( #[cfg(feature = $feat)] )?
			impl<'de> Deserialize<'de> for $name<$t> {
				fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
				where
					D: serde::Deserializer<'de>,
				{
					Ok($name($deser(deserializer)?))
				}
			}

			$( #[cfg(feature = $feat)] )?
			impl Serialize for $name<$t> {
				fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
					$ser(serializer, &self.0)
				}
			}

			$( #[cfg(feature = $feat)] )?
			paste::paste! {
				#[test]
				fn [<test_ $name:lower _ $t:lower _serde>]() {
					let x = $name::<$t>::from_uint(567);
					let json = serde_json::to_value(&x).unwrap();
					assert_eq!(json.as_i64().unwrap(), 567);
					let x_parsed: $name<$t> = serde_json::from_value(json).unwrap();
					assert_eq!(x, x_parsed);
				}

				#[test]
				#[cfg(feature = "schemars")]
				fn [<test_ $name:lower _ $t:lower _schemars>]() {
					fn mk_default() -> $name<$t> {
						$name::<$t>::from_uint(567)
					}
					#[derive(schemars::JsonSchema)]
					struct TestStruct {
						#[serde(default = "mk_default")]
						_duration: $name<$t>,
					}
					let schema = schemars::schema_for!(TestStruct).as_value()["properties"]["_duration"].clone();
					assert_eq!(schema["default"], 567);
					assert_eq!(schema["type"], "integer");
				}
			}
		)*
	};
}

#[cfg(feature = "time")]
use time::Duration as TimeDuration;

define_generic_wrapper! {
	"`Duration` wrapper with [`Deserialize`] impl",
	Seconds:

	{
		StdDuration,
		|deserializer| u64::deserialize(deserializer).map(StdDuration::from_secs),
		|serializer: S, x: &StdDuration| serializer.serialize_u64(x.as_secs())
	},

	feature "time";
	{
		TimeDuration,
		|deserializer| i64::deserialize(deserializer).map(TimeDuration::seconds),
		|serializer: S, x: &TimeDuration| serializer.serialize_i64(x.whole_seconds())
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
	pub const fn from_uint(s: u32) -> Self {
		Self::from_int(s as i64)
	}

	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_int(s: i64) -> Self {
		Seconds(TimeDuration::seconds(s))
	}
}

define_generic_wrapper! {
	"Helper wrapper to use in configs to deserialize durations from minutes",
	Minutes:

	{
		StdDuration,
		|deserializer| u64::deserialize(deserializer).map(|x| StdDuration::from_secs(x * 60)),
		|serializer: S, x: &StdDuration| serializer.serialize_u64(x.as_secs() / 60)
	},

	feature "time";
	{
		TimeDuration,
		|deserializer| i64::deserialize(deserializer).map(TimeDuration::minutes),
		|serializer: S, x: &TimeDuration| serializer.serialize_i64(x.whole_minutes())
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
	pub const fn from_uint(m: u32) -> Self {
		Self::from_int(m as i64)
	}

	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_int(m: i64) -> Self {
		Minutes(TimeDuration::minutes(m))
	}
}

define_generic_wrapper! {
	"Helper wrapper to use in configs to deserialize durations from hours",
	Hours:

	{
		StdDuration,
		|deserializer| u64::deserialize(deserializer).map(|x| StdDuration::from_secs(x * 60 * 60)),
		|serializer: S, x: &StdDuration| serializer.serialize_u64(x.as_secs() / 60 / 60)
	},

	feature "time";
	{
		TimeDuration,
		|deserializer| i64::deserialize(deserializer).map(TimeDuration::hours),
		|serializer: S, x: &TimeDuration| serializer.serialize_i64(x.whole_hours())
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
	pub const fn from_uint(h: u32) -> Self {
		Self::from_int(h as i64)
	}

	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_int(h: i64) -> Self {
		Hours(TimeDuration::hours(h))
	}
}

define_generic_wrapper! {
	"Helper wrapper to use in configs to deserialize durations from milliseconds",
	Ms:

	{
		StdDuration,
		|deserializer| u64::deserialize(deserializer).map(StdDuration::from_millis),
		|serializer: S, x: &StdDuration| serializer.serialize_u128(x.as_millis())
	},

	feature "time";
	{
		TimeDuration,
		|deserializer| i64::deserialize(deserializer).map(TimeDuration::milliseconds),
		|serializer: S, x: &TimeDuration| serializer.serialize_i128(x.whole_milliseconds())
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
	pub const fn from_uint(ms: u32) -> Self {
		Self::from_int(ms as i64)
	}

	#[allow(missing_docs)]
	#[must_use]
	pub const fn from_int(ms: i64) -> Self {
		Ms(TimeDuration::milliseconds(ms))
	}
}
