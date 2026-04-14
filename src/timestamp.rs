// SPDX-FileCopyrightText: 2026 Famedly GmbH (info@famedly.com)
//
// SPDX-License-Identifier: Apache-2.0

//! Wrapper over [`OffsetDateTime`] with RFC3339 JSON representation

#![allow(missing_docs, unused_qualifications)]

use core::{fmt, str};

#[cfg(feature = "schemars")]
use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

/// Timestamp with RFC3339 JSON representation
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
pub struct Timestamp {
	#[serde(with = "time::serde::rfc3339")]
	ts: OffsetDateTime,
}

impl AsRef<OffsetDateTime> for Timestamp {
	fn as_ref(&self) -> &OffsetDateTime {
		&self.ts
	}
}

impl std::ops::Deref for Timestamp {
	type Target = OffsetDateTime;
	fn deref(&self) -> &Self::Target {
		&self.ts
	}
}

impl fmt::Display for Timestamp {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		use time::format_description::well_known::Rfc3339;
		let ts = self.ts.format(&Rfc3339).map_err(|_| fmt::Error)?;
		write!(f, "{}", ts)
	}
}

impl str::FromStr for Timestamp {
	type Err = time::error::Parse;
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		use time::format_description::well_known::Rfc3339;
		let ts = time::OffsetDateTime::parse(s, &Rfc3339)?;
		Ok(Self { ts })
	}
}

#[cfg(feature = "schemars")]
impl JsonSchema for Timestamp {
	fn schema_name() -> std::borrow::Cow<'static, str> {
		"Timestamp".into()
	}
	fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
		schemars::json_schema!({
			"type": "string",
			"format": "date-time",
		})
	}
	fn inline_schema() -> bool {
		true
	}
}

impl From<OffsetDateTime> for Timestamp {
	fn from(ts: OffsetDateTime) -> Self {
		Self { ts }
	}
}

impl From<Timestamp> for OffsetDateTime {
	fn from(ts: Timestamp) -> Self {
		ts.ts
	}
}
