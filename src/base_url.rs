use std::ops::Deref;

use serde::{de::Error, Deserialize, Deserializer, Serialize};
use thiserror::Error;
use url::Url;

use crate::GenericCombinators;

/// A wrapper over [Url] to deserialize all urls as base url (add trailing slash
/// if necessary). It is intended as drop-in replacement for [Url], but also
/// provides [BaseUrl::append_path] method with stricter guarantees.
/// ```
/// # use famedly_rust_utils::BaseUrl;
/// #[derive(serde::Deserialize)]
/// struct Foo {
/// 	base_url: BaseUrl
/// }
///
/// let foo: Foo = serde_json::from_value(serde_json::json!({"base_url": "http://example.com"})).unwrap();
/// assert_eq!(foo.base_url.as_str(), "http://example.com/");
/// ```
#[derive(Debug, PartialEq, Eq, Clone, Deserialize, Serialize)]
#[repr(transparent)]
#[serde(transparent)]
pub struct BaseUrl {
	#[serde(deserialize_with = "deserialize_url_with_trailing_slash")]
	url: Url,
}

impl BaseUrl {
	/// Append path segments to the path of a Url, escaping if necessary.
	/// Source: https://github.com/servo/rust-url/pull/934.
	///
	/// This differs from Url::join in that it is insensitive to trailing
	/// slashes in the url and leading slashes in the passed string. See
	/// documentation of Url::join for discussion of this subtlety. Also, this
	/// function cannot change any part of the Url other than the path.
	/// ```
	/// # use famedly_rust_utils::BaseUrl;
	/// # use url::Url;
	/// let mut my_url: BaseUrl = Url::parse("http://www.example.com/api/v1")
	/// 	.unwrap()
	/// 	.try_into()
	/// 	.unwrap();
	/// my_url.append_path("system").unwrap();
	/// my_url.append_path("status").unwrap();
	/// assert_eq!(my_url.as_str(), "http://www.example.com/api/v1/system/status");
	/// ```
	/// Fails if the Url is cannot-be-a-base.
	#[allow(clippy::result_unit_err)]
	#[inline]
	pub fn append_path(&mut self, path: impl AsRef<str>) -> Result<(), ()> {
		let mut path_segments_mut = self.url.path_segments_mut()?;
		path_segments_mut.pop_if_empty();
		let path = path.as_ref();
		path_segments_mut.extend(path.strip_prefix('/').unwrap_or(path).split('/'));
		Ok(())
	}
}

impl std::fmt::Display for BaseUrl {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
		self.url.fmt(f)
	}
}

impl TryFrom<Url> for BaseUrl {
	type Error = ();
	fn try_from(url: Url) -> Result<Self, Self::Error> {
		if url.cannot_be_a_base() {
			return Err(());
		}
		Ok(BaseUrl { url: url.mutate(add_trailing_slash) })
	}
}

/// Parsing error for [BaseUrl]
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BaseUrlParseError {
	/// Parsing error for [Url]
	#[error("Url parse error: {0}")]
	UrlParseError(#[from] url::ParseError),
	/// A case when a [Url] is valid but it can't be a base
	#[error("Url cannot be a base")]
	IsNotBaseUrl,
}

impl std::str::FromStr for BaseUrl {
	type Err = BaseUrlParseError;

	#[inline]
	fn from_str(input: &str) -> Result<BaseUrl, Self::Err> {
		Url::parse(input)?.try_into().map_err(|_| BaseUrlParseError::IsNotBaseUrl)
	}
}

impl From<BaseUrl> for Url {
	fn from(url: BaseUrl) -> Self {
		url.url
	}
}

impl Deref for BaseUrl {
	type Target = Url;
	fn deref(&self) -> &Self::Target {
		&self.url
	}
}

/// Add trailing slash to [Url]
fn add_trailing_slash(url: &mut Url) {
	if !url.path().ends_with('/') {
		url.set_path(&format!("{}/", url.path()));
	}
}

/// Deserialize [Url] with trailing slash
fn deserialize_url_with_trailing_slash<'de, D>(deserializer: D) -> Result<Url, D::Error>
where
	D: Deserializer<'de>,
{
	let url = Url::deserialize(deserializer)?;
	if url.cannot_be_a_base() {
		return Err(Error::custom("Url cannot be a base"));
	}
	Ok(url.mutate(add_trailing_slash))
}

#[test]
fn test_deserialization() {
	#[derive(Deserialize)]
	struct Foo {
		base_url: BaseUrl,
	}

	let foo: Foo =
		serde_json::from_value(serde_json::json!({"base_url": "http://example.com"})).unwrap();
	assert_eq!(foo.base_url.as_str(), "http://example.com/");
}

#[test]
fn test_append_path() {
	let mut my_url: BaseUrl =
		Url::parse("http://www.example.com/api/v1").unwrap().try_into().unwrap();
	my_url.append_path("system").unwrap();
	my_url.append_path("status").unwrap();
	assert_eq!(my_url.as_str(), "http://www.example.com/api/v1/system/status");
}
