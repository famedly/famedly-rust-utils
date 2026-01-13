// SPDX-FileCopyrightText: 2025 Famedly GmbH (info@famedly.com)
//
// SPDX-License-Identifier: Apache-2.0

//! Non-empty string and collection types for enforcing invariants at API boundaries.
//!
//! This module provides wrapper types that guarantee non-empty values through
//! deserialization-time validation, helping to fail fast when invalid inputs
//! are provided to API endpoints.
//!
//! # Examples
//!
//! ## Using NonEmptyString
//!
//! ```
//! # use famedly_rust_utils::NonEmptyString;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct UserRequest {
//!     username: NonEmptyString,
//! }
//!
//! // Valid input deserializes successfully
//! let valid = serde_json::from_str::<UserRequest>(r#"{"username": "alice"}"#).unwrap();
//! assert_eq!(valid.username.as_str(), "alice");
//!
//! // Empty string is rejected at deserialization time
//! let invalid = serde_json::from_str::<UserRequest>(r#"{"username": ""}"#);
//! assert!(invalid.is_err());
//! ```
//!
//! ## Using TrimmedNonEmptyString
//!
//! ```
//! # use famedly_rust_utils::TrimmedNonEmptyString;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct CommentRequest {
//!     text: TrimmedNonEmptyString,
//! }
//!
//! // Whitespace is trimmed automatically
//! let valid = serde_json::from_str::<CommentRequest>(r#"{"text": "  hello  "}"#).unwrap();
//! assert_eq!(valid.text.as_str(), "hello");
//!
//! // Whitespace-only strings are rejected
//! let invalid = serde_json::from_str::<CommentRequest>(r#"{"text": "   "}"#);
//! assert!(invalid.is_err());
//! ```
//!
//! ## Using NonEmptyVec
//!
//! ```
//! # use famedly_rust_utils::NonEmptyVec;
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct BatchRequest {
//!     items: NonEmptyVec<String>,
//! }
//!
//! // Non-empty lists deserialize successfully
//! let valid = serde_json::from_str::<BatchRequest>(r#"{"items": ["a", "b"]}"#).unwrap();
//! assert_eq!(valid.items.len(), 2);
//!
//! // Empty lists are rejected at deserialization time
//! let invalid = serde_json::from_str::<BatchRequest>(r#"{"items": []}"#);
//! assert!(invalid.is_err());
//! ```

use std::ops::Deref;

#[cfg(feature = "serde")]
use serde::{de::Error, Deserialize, Deserializer, Serialize};
use thiserror::Error;

/// Error type for non-empty validation failures.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NonEmptyError {
	/// The string is empty
	#[error("string must be non-empty")]
	EmptyString,
	/// The string contains only whitespace after trimming
	#[error("string must contain non-whitespace characters")]
	BlankString,
	/// The collection is empty
	#[error("collection must be non-empty")]
	EmptyCollection,
}

/// A non-empty string wrapper that rejects empty strings during deserialization.
///
/// This type guarantees that the contained string is not empty, making it
/// suitable for API fields that must reject empty input at the boundary.
///
/// # Examples
///
/// ```
/// # use famedly_rust_utils::NonEmptyString;
/// # use serde::Deserialize;
/// #[derive(Deserialize)]
/// struct Config {
///     api_key: NonEmptyString,
/// }
///
/// // Valid deserialization
/// let config: Config = serde_json::from_str(r#"{"api_key": "abc123"}"#).unwrap();
/// assert_eq!(config.api_key.as_str(), "abc123");
///
/// // Invalid deserialization fails
/// assert!(serde_json::from_str::<Config>(r#"{"api_key": ""}"#).is_err());
/// ```
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(transparent)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct NonEmptyString {
	value: String,
}

impl NonEmptyString {
	/// Creates a new `NonEmptyString` from a string.
	///
	/// Returns an error if the string is empty.
	///
	/// # Examples
	///
	/// ```
	/// # use famedly_rust_utils::NonEmptyString;
	/// let s = NonEmptyString::new("hello".to_string()).unwrap();
	/// assert_eq!(s.as_str(), "hello");
	///
	/// assert!(NonEmptyString::new("".to_string()).is_err());
	/// ```
	#[inline]
	pub fn new(s: String) -> Result<Self, NonEmptyError> {
		if s.is_empty() {
			Err(NonEmptyError::EmptyString)
		} else {
			Ok(NonEmptyString { value: s })
		}
	}

	/// Returns the inner string as a string slice.
	#[inline]
	#[must_use]
	pub fn as_str(&self) -> &str {
		&self.value
	}

	/// Consumes the wrapper and returns the inner string.
	#[inline]
	#[must_use]
	pub fn into_inner(self) -> String {
		self.value
	}
}

impl std::str::FromStr for NonEmptyString {
	type Err = NonEmptyError;

	#[inline]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::new(s.to_owned())
	}
}

impl std::fmt::Display for NonEmptyString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.value.fmt(f)
	}
}

impl AsRef<str> for NonEmptyString {
	fn as_ref(&self) -> &str {
		&self.value
	}
}

impl Deref for NonEmptyString {
	type Target = str;
	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for NonEmptyString {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let s = String::deserialize(deserializer)?;
		Self::new(s).map_err(D::Error::custom)
	}
}

/// A non-empty string wrapper that trims whitespace and rejects blank strings
/// during deserialization.
///
/// This type guarantees that the contained string is not empty after trimming
/// whitespace, making it suitable for API fields that must reject blank input.
///
/// # Examples
///
/// ```
/// # use famedly_rust_utils::TrimmedNonEmptyString;
/// # use serde::Deserialize;
/// #[derive(Deserialize)]
/// struct Comment {
///     text: TrimmedNonEmptyString,
/// }
///
/// // Whitespace is trimmed
/// let comment: Comment = serde_json::from_str(r#"{"text": "  hello  "}"#).unwrap();
/// assert_eq!(comment.text.as_str(), "hello");
///
/// // Blank strings are rejected
/// assert!(serde_json::from_str::<Comment>(r#"{"text": "   "}"#).is_err());
/// ```
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(Serialize))]
#[repr(transparent)]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct TrimmedNonEmptyString {
	value: String,
}

impl TrimmedNonEmptyString {
	/// Creates a new `TrimmedNonEmptyString` from a string.
	///
	/// The string is trimmed and then validated to be non-empty.
	///
	/// # Examples
	///
	/// ```
	/// # use famedly_rust_utils::TrimmedNonEmptyString;
	/// let s = TrimmedNonEmptyString::new("  hello  ".to_string()).unwrap();
	/// assert_eq!(s.as_str(), "hello");
	///
	/// assert!(TrimmedNonEmptyString::new("   ".to_string()).is_err());
	/// ```
	#[inline]
	pub fn new(s: String) -> Result<Self, NonEmptyError> {
		let trimmed = s.trim().to_owned();
		if trimmed.is_empty() {
			Err(NonEmptyError::BlankString)
		} else {
			Ok(TrimmedNonEmptyString { value: trimmed })
		}
	}

	/// Returns the inner string as a string slice.
	#[inline]
	#[must_use]
	pub fn as_str(&self) -> &str {
		&self.value
	}

	/// Consumes the wrapper and returns the inner string.
	#[inline]
	#[must_use]
	pub fn into_inner(self) -> String {
		self.value
	}
}

impl std::str::FromStr for TrimmedNonEmptyString {
	type Err = NonEmptyError;

	#[inline]
	fn from_str(s: &str) -> Result<Self, Self::Err> {
		Self::new(s.to_owned())
	}
}

impl std::fmt::Display for TrimmedNonEmptyString {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.value.fmt(f)
	}
}

impl AsRef<str> for TrimmedNonEmptyString {
	fn as_ref(&self) -> &str {
		&self.value
	}
}

impl Deref for TrimmedNonEmptyString {
	type Target = str;
	fn deref(&self) -> &Self::Target {
		&self.value
	}
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for TrimmedNonEmptyString {
	fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
		let s = String::deserialize(deserializer)?;
		Self::new(s).map_err(D::Error::custom)
	}
}

/// A non-empty vector type alias.
///
/// This is a re-export of [`nonempty::NonEmpty`] specialized for `Vec<T>`,
/// providing compile-time guarantees that a collection contains at least one element.
///
/// The type has full serde support and will reject empty arrays during deserialization.
///
/// # Examples
///
/// ```
/// # use famedly_rust_utils::NonEmptyVec;
/// # use serde::Deserialize;
/// #[derive(Deserialize)]
/// struct Request {
///     ids: NonEmptyVec<u32>,
/// }
///
/// // Non-empty arrays deserialize successfully
/// let req: Request = serde_json::from_str(r#"{"ids": [1, 2, 3]}"#).unwrap();
/// assert_eq!(req.ids.len(), 3);
///
/// // Empty arrays are rejected
/// assert!(serde_json::from_str::<Request>(r#"{"ids": []}"#).is_err());
/// ```
pub type NonEmptyVec<T> = nonempty::NonEmpty<T>;

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_non_empty_string_new_valid() {
		let s = NonEmptyString::new("hello".to_owned()).unwrap();
		assert_eq!(s.as_str(), "hello");
	}

	#[test]
	fn test_non_empty_string_new_empty() {
		let err = NonEmptyString::new("".to_owned()).unwrap_err();
		assert_eq!(err, NonEmptyError::EmptyString);
	}

	#[test]
	fn test_non_empty_string_from_str_valid() {
		let s: NonEmptyString = "hello".parse().unwrap();
		assert_eq!(s.as_str(), "hello");
	}

	#[test]
	fn test_non_empty_string_from_str_empty() {
		let err: NonEmptyError = "".parse::<NonEmptyString>().unwrap_err();
		assert_eq!(err, NonEmptyError::EmptyString);
	}

	#[test]
	fn test_non_empty_string_display() {
		let s = NonEmptyString::new("hello".to_owned()).unwrap();
		assert_eq!(format!("{}", s), "hello");
	}

	#[test]
	fn test_non_empty_string_deref() {
		let s = NonEmptyString::new("hello".to_owned()).unwrap();
		assert_eq!(s.len(), 5);
		assert!(s.starts_with("he"));
	}

	#[cfg(feature = "serde")]
	#[test]
	fn test_non_empty_string_deserialize_valid() {
		#[derive(serde::Deserialize)]
		struct TestStruct {
			field: NonEmptyString,
		}

		let json = r#"{"field": "value"}"#;
		let result: TestStruct = serde_json::from_str(json).unwrap();
		assert_eq!(result.field.as_str(), "value");
	}

	#[cfg(feature = "serde")]
	#[test]
	fn test_non_empty_string_deserialize_empty() {
		#[derive(Debug, serde::Deserialize)]
		struct TestStruct {
			field: NonEmptyString,
		}

		let json = r#"{"field": ""}"#;
		let result = serde_json::from_str::<TestStruct>(json);
		assert!(result.is_err());
		let err_msg = result.unwrap_err().to_string();
		assert!(err_msg.contains("non-empty") || err_msg.contains("must"));
	}

	#[cfg(feature = "serde")]
	#[test]
	fn test_non_empty_string_serialize() {
		#[derive(serde::Serialize)]
		struct TestStruct {
			field: NonEmptyString,
		}

		let s = TestStruct { field: NonEmptyString::new("value".to_owned()).unwrap() };
		let json = serde_json::to_string(&s).unwrap();
		assert_eq!(json, r#"{"field":"value"}"#);
	}

	#[test]
	fn test_trimmed_non_empty_string_new_valid() {
		let s = TrimmedNonEmptyString::new("  hello  ".to_owned()).unwrap();
		assert_eq!(s.as_str(), "hello");
	}

	#[test]
	fn test_trimmed_non_empty_string_new_blank() {
		let err = TrimmedNonEmptyString::new("   ".to_owned()).unwrap_err();
		assert_eq!(err, NonEmptyError::BlankString);
	}

	#[test]
	fn test_trimmed_non_empty_string_new_empty() {
		let err = TrimmedNonEmptyString::new("".to_owned()).unwrap_err();
		assert_eq!(err, NonEmptyError::BlankString);
	}

	#[test]
	fn test_trimmed_non_empty_string_from_str_valid() {
		let s: TrimmedNonEmptyString = "  hello  ".parse().unwrap();
		assert_eq!(s.as_str(), "hello");
	}

	#[test]
	fn test_trimmed_non_empty_string_from_str_blank() {
		let err: NonEmptyError = "   ".parse::<TrimmedNonEmptyString>().unwrap_err();
		assert_eq!(err, NonEmptyError::BlankString);
	}

	#[test]
	fn test_trimmed_non_empty_string_display() {
		let s = TrimmedNonEmptyString::new("  hello  ".to_owned()).unwrap();
		assert_eq!(format!("{}", s), "hello");
	}

	#[test]
	fn test_trimmed_non_empty_string_deref() {
		let s = TrimmedNonEmptyString::new("  hello  ".to_owned()).unwrap();
		assert_eq!(s.len(), 5);
		assert!(s.starts_with("he"));
	}

	#[cfg(feature = "serde")]
	#[test]
	fn test_trimmed_non_empty_string_deserialize_valid() {
		#[derive(serde::Deserialize)]
		struct TestStruct {
			field: TrimmedNonEmptyString,
		}

		let json = r#"{"field": "  value  "}"#;
		let result: TestStruct = serde_json::from_str(json).unwrap();
		assert_eq!(result.field.as_str(), "value");
	}

	#[cfg(feature = "serde")]
	#[test]
	fn test_trimmed_non_empty_string_deserialize_blank() {
		#[derive(Debug, serde::Deserialize)]
		struct TestStruct {
			field: TrimmedNonEmptyString,
		}

		let json = r#"{"field": "   "}"#;
		let result = serde_json::from_str::<TestStruct>(json);
		assert!(result.is_err());
		let err_msg = result.unwrap_err().to_string();
		assert!(err_msg.contains("whitespace") || err_msg.contains("blank") || err_msg.contains("must"));
	}

	#[cfg(feature = "serde")]
	#[test]
	fn test_trimmed_non_empty_string_serialize() {
		#[derive(serde::Serialize)]
		struct TestStruct {
			field: TrimmedNonEmptyString,
		}

		let s = TestStruct {
			field: TrimmedNonEmptyString::new("  value  ".to_owned()).unwrap(),
		};
		let json = serde_json::to_string(&s).unwrap();
		assert_eq!(json, r#"{"field":"value"}"#);
	}

	#[cfg(feature = "serde")]
	#[test]
	fn test_non_empty_vec_deserialize_valid() {
		#[derive(serde::Deserialize)]
		struct TestStruct {
			items: NonEmptyVec<String>,
		}

		let json = r#"{"items": ["a", "b", "c"]}"#;
		let result: TestStruct = serde_json::from_str(json).unwrap();
		assert_eq!(result.items.len(), 3);
		assert_eq!(result.items.head, "a");
	}

	#[cfg(feature = "serde")]
	#[test]
	fn test_non_empty_vec_deserialize_empty() {
		#[derive(Debug, serde::Deserialize)]
		struct TestStruct {
			items: NonEmptyVec<String>,
		}

		let json = r#"{"items": []}"#;
		let result = serde_json::from_str::<TestStruct>(json);
		assert!(result.is_err());
		let err_msg = result.unwrap_err().to_string();
		assert!(err_msg.contains("empty") || err_msg.contains("must"));
	}

	#[cfg(feature = "serde")]
	#[test]
	fn test_non_empty_vec_serialize() {
		#[derive(serde::Serialize)]
		struct TestStruct {
			items: NonEmptyVec<u32>,
		}

		let vec = nonempty::nonempty![1, 2, 3];
		let s = TestStruct { items: vec };
		let json = serde_json::to_string(&s).unwrap();
		assert_eq!(json, r#"{"items":[1,2,3]}"#);
	}

	#[test]
	fn test_non_empty_vec_construction() {
		// Test using the nonempty! macro
		let vec = nonempty::nonempty![1, 2, 3];
		assert_eq!(vec.len(), 3);
		assert_eq!(vec.head, 1);

		// Test using from_vec
		let regular_vec = vec![4, 5, 6];
		let non_empty = nonempty::NonEmpty::from_vec(regular_vec).unwrap();
		assert_eq!(non_empty.len(), 3);
		assert_eq!(non_empty.head, 4);

		// Test from_vec with empty vec
		let empty_vec: Vec<i32> = vec![];
		let result = nonempty::NonEmpty::from_vec(empty_vec);
		assert!(result.is_none());
	}
}
