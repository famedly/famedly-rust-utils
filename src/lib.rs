//! Famedly rust utils

/// Workaround on [`url::Url::join` behavior](https://github.com/servo/rust-url/issues/333)
mod base_url;
pub mod duration;
/// [serde::Deserialize] impl for [tracing::level_filters::LevelFilter]
mod level_filter;
#[cfg(feature = "reqwest")]
/// Helpers for [reqwest]
pub mod reqwest;
#[cfg(feature = "telemetry")]
/// Function to setup the telemetry tools
pub mod telemetry;

pub use base_url::{BaseUrl, BaseUrlParseError};
pub use level_filter::LevelFilter;

/// Generic combinators
pub trait GenericCombinators {
	/// Convenience method to cast everything into `()`. For example:
	/// ```
	/// async fn get_int() -> Result<u8, String> {
	/// 	Ok(3)
	/// }
	///
	/// // without `ignore`:
	/// async fn f() -> Result<(), String> {
	/// 	let _ = get_int().await?;
	/// 	Ok(())
	/// }
	///
	/// # use famedly_rust_utils::GenericCombinators;
	/// // with `ignore`:
	/// async fn g() -> Result<(), String> {
	/// 	Ok(get_int().await?.ignore())
	/// }
	/// ```
	fn ignore(&self);

	/// Helper method to isolate mutations and avoid needless `mut` bindings
	/// ```
	/// # use famedly_rust_utils::GenericCombinators;
	/// let path_buf = std::path::PathBuf::from("/")
	/// 	.mutate(|p| p.push("etc"))
	/// 	.mutate(|p| p.push("ssh"));
	/// assert_eq!(path_buf.to_str().unwrap(), "/etc/ssh");
	/// ```
	#[must_use]
	fn mutate(self, f: impl FnOnce(&mut Self)) -> Self;

	/// Helper method to inline optional steps in chains
	/// ```
	/// # use famedly_rust_utils::GenericCombinators;
	/// # #[derive(Debug)]
	/// struct S(usize);
	/// impl S {
	/// 	fn new() -> Self {
	/// 		S(0)
	/// 	}
	/// 	fn inc(self) -> Self {
	/// 		S(self.0 + 1)
	/// 	}
	/// }
	/// assert_eq!(S::new().inc().chain_if(false, S::inc).0, 1);
	/// assert_eq!(S::new().inc().chain_if(true, S::inc).0, 2);
	/// ```
	#[must_use]
	fn chain_if(self, p: bool, f: impl FnOnce(Self) -> Self) -> Self
	where
		Self: Sized;

	/// Helper method to inline optional steps in chains. Analogous to
	/// [GenericCombinators::chain_if]
	/// ```
	/// # use famedly_rust_utils::GenericCombinators;
	/// # #[derive(Debug)]
	/// struct S(usize);
	/// impl S {
	/// 	fn new() -> Self {
	/// 		S(0)
	/// 	}
	/// 	fn add(self, x: usize) -> Self {
	/// 		S(self.0 + x)
	/// 	}
	/// }
	/// assert_eq!(S::new().add(1).chain_opt(None, S::add).0, 1);
	/// assert_eq!(S::new().add(1).chain_opt(Some(1), S::add).0, 2);
	/// ```
	#[must_use]
	fn chain_opt<X>(self, o: Option<X>, f: impl FnOnce(Self, X) -> Self) -> Self
	where
		Self: Sized;
}

impl<A> GenericCombinators for A {
	#[inline]
	fn ignore(&self) {}

	#[inline]
	fn mutate(mut self, f: impl FnOnce(&mut Self)) -> Self {
		f(&mut self);
		self
	}

	#[inline]
	fn chain_if(self, p: bool, f: impl FnOnce(Self) -> Self) -> Self {
		if p {
			f(self)
		} else {
			self
		}
	}

	#[inline]
	fn chain_opt<X>(self, o: Option<X>, f: impl FnOnce(Self, X) -> Self) -> Self {
		if let Some(x) = o {
			f(self, x)
		} else {
			self
		}
	}
}

/// Helper function to convert values to `()`
/// ```
/// async fn get_int() -> Result<u8, String> {
/// 	Ok(3)
/// }
///
/// // without `ignore`:
/// async fn f() -> Result<(), String> {
/// 	let _ = get_int().await?;
/// 	Ok(())
/// }
///
/// # use famedly_rust_utils::ignore;
///
/// // with `ignore`:
/// async fn g() -> Result<(), String> {
/// 	get_int().await.map(ignore)
/// }
/// ```
#[inline]
pub fn ignore<X>(_: X) {}

/// Extension to [Iterator]
pub trait IteratorExt: Iterator {
	/// Helper function for external types that lack `FromIterator`
	/// implementation
	/// ```
	/// # use famedly_rust_utils::IteratorExt;
	/// assert_eq!((1..=3).generic_collect(Vec::new(), Vec::push), vec![1, 2, 3]);
	/// ```
	fn generic_collect<T>(self, empty: T, f: impl Fn(&mut T, Self::Item)) -> T;
}

impl<I: Iterator> IteratorExt for I {
	#[inline]
	fn generic_collect<T>(self, mut acc: T, f: impl Fn(&mut T, Self::Item)) -> T {
		self.for_each(|x| f(&mut acc, x));
		acc
	}
}

#[test]
fn test_ignore() -> Result<(), String> {
	let some_res: Result<u8, String> = Ok(3);
	Ok(some_res?.ignore())
}

#[test]
fn test_mutate() {
	let path_buf =
		std::path::PathBuf::from("/").mutate(|p| p.push("etc")).mutate(|p| p.push("ssh"));
	assert_eq!(path_buf.to_str().unwrap(), "/etc/ssh");
}

#[test]
fn test_chain_if() {
	#[derive(Debug)]
	struct S(usize);
	impl S {
		fn new() -> Self {
			S(0)
		}
		fn inc(self) -> Self {
			S(self.0 + 1)
		}
	}
	assert_eq!(S::new().inc().chain_if(false, S::inc).0, 1);
	assert_eq!(S::new().inc().chain_if(true, S::inc).0, 2);
}

#[test]
fn test_chain_opt() {
	#[derive(Debug)]
	struct S(usize);
	impl S {
		fn new() -> Self {
			S(0)
		}
		fn add(self, x: usize) -> Self {
			S(self.0 + x)
		}
	}
	assert_eq!(S::new().add(1).chain_opt(None, S::add).0, 1);
	assert_eq!(S::new().add(1).chain_opt(Some(1), S::add).0, 2);
}

#[test]
fn test_generic_collect() {
	assert_eq!((1..=3).generic_collect(Vec::new(), Vec::push), vec![1, 2, 3]);
}

#[doc(hidden)]
macro_rules! define_generic_wrapper {
	($doc:expr, $name:ident: $( $(feature $feat:expr; )? { $t:ty, $deser:expr, $($ser:expr)? }),*) => {
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
			$( #[cfg(feature = $feat)] )?
			impl<'de> serde::Deserialize<'de> for $name<$t> {
				fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
				where
					D: serde::Deserializer<'de>,
				{
					Ok($name($deser(deserializer)?))
				}
			}
		)*
	};
}

pub(crate) use define_generic_wrapper;
