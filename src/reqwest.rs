use std::{fmt, future::Future};

/// Wrapper around [reqwest::Error] with optional response body
#[derive(Debug, thiserror::Error)]
pub struct ReqwestErrorWithBody {
	/// Error from [reqwest]
	pub error: reqwest::Error,
	/// Optional response body
	pub body: Option<String>,
}

impl From<reqwest::Error> for ReqwestErrorWithBody {
	fn from(error: reqwest::Error) -> ReqwestErrorWithBody {
		Self { error, body: None }
	}
}

impl fmt::Display for ReqwestErrorWithBody {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{} with body {}", self.error, self.body.as_deref().unwrap_or("<no body>"))
	}
}

/// An alternative to [reqwest::Resnpose::error_for_status] that also returns
/// optional response body
/// ```no_run
/// # use famedly_rust_utils::reqwest::*;
/// # async fn mk_req() -> Result<(), ReqwestErrorWithBody> {
/// reqwest::get("http://invalid.example")
/// 	.await?
/// 	.error_for_status_with_body()
/// 	.await?;
/// # Ok(())
/// # }
/// ```
pub trait ErrorForStatusWithBody {
	// Using explicit `impl Future` syntax here instead of `async_fn_in_trait` to
	// make it `Send`
	#[allow(missing_docs)]
	fn error_for_status_with_body(
		self,
	) -> impl Future<Output = Result<reqwest::Response, ReqwestErrorWithBody>> + Send;
}

impl ErrorForStatusWithBody for reqwest::Response {
	#[allow(clippy::manual_async_fn)]
	fn error_for_status_with_body(
		self,
	) -> impl Future<Output = Result<reqwest::Response, ReqwestErrorWithBody>> + Send {
		async {
			if let Err(error) = self.error_for_status_ref() {
				Err(ReqwestErrorWithBody { error, body: self.text().await.ok() })
			} else {
				Ok(self)
			}
		}
	}
}
