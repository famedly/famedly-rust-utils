// SPDX-FileCopyrightText: 2025 Famedly GmbH (info@famedly.com)
//
// SPDX-License-Identifier: Apache-2.0

//! Generic configuration parsers.
use std::path::Path;

use figment::{providers::Format, Figment};
use serde::de::DeserializeOwned;

const ANSI_RED: &str = "\x1b[1;31m";
const ANSI_GREEN: &str = "\x1b[1;32m";
const ANSI_YELLOW: &str = "\x1b[1;33m";
const ANSI_RESET: &str = "\x1b[0m";

/// Standardized Famedly configuration file parsing using figment.
///
/// Halts the process with a user-oriented error message if the
/// configuration fails to parse; this prevents any issues with
/// unconfigured logging.
///
/// Parsing combines values from environment variables prefixed with
/// `env_prefix`, as well as `config.yml` and `config.yaml` files in
/// the current working directory. The priority of each configuration
/// location is:
///
///   environment > config.yaml > config.yml
///
/// Nested values can be separated with `__` in environment variables.
///
/// IMPORTANT
/// =========
/// Since failure terminates the process, no destructors on the
/// current or other threads are executed on failure.
///
/// This function should therefore never be called after any real
/// business logic has started. Config parsing *should* finish
/// beforehand, so this is ok, but user beware.
#[must_use]
pub fn parse_config<C: DeserializeOwned>(env_prefix: &str) -> C {
	try_parse_config(env_prefix).unwrap_or_else(|error| {
		print_parse_config_errors(env_prefix, error);
		std::process::exit(1);
	})
}

#[allow(clippy::print_stderr)]
fn print_parse_config_errors(env_prefix: &str, error: Box<figment::Error>) {
	fn print_note(note: impl AsRef<str>) {
		eprintln!("\n{}note{}: {}", ANSI_GREEN, ANSI_RESET, note.as_ref());
	}

	eprintln!("{}error{}: invalid configuration:", ANSI_RED, ANSI_RESET,);

	for error in *error {
		eprintln!("- {error}");
	}

	let env_var_typo_heuristic = std::env::vars().any(|(var, _)| var.starts_with(env_prefix));
	let missing_config_file_heuristic =
		!(Path::new("./config.yml").exists() || Path::new("./config.yaml").exists());

	if missing_config_file_heuristic {
		print_note("neither `./config.yaml` nor `./config.yml` could be found; ensure that you have read permissions and that the filename is correct");
	};

	if env_var_typo_heuristic {
		print_note(format!("an environment variable starting with with `{env_prefix}` was found; check any variable names for typos"));
	}
}

/// Standardized Famedly configuration file parsing, but does *not*
/// call `process::exit` on failure. See [`parse_config`] for other
/// details.
///
/// Note that if you use this function instead of the terminating one
/// that figment will not warn users for completely missing
/// configuration, which can be confusing if a config file cannot be
/// read or is misnamed.
///
/// See [`print_parse_config_errors`] for further edge cases.
#[allow(clippy::print_stderr)]
pub fn try_parse_config<C: DeserializeOwned>(env_prefix: &str) -> Result<C, Box<figment::Error>> {
	// TODO: Starting with version 0.10.20, figment will support
	// doing this with relative file paths by using
	// `.search(false)`.
	//
	// At that point we'll be able to remove this ugly hack and still
	// resolve only config files in the CWD - the default behavior is
	// to bubble up the search to parent directories.
	if let Ok(cwd) = std::env::current_dir() {
		Figment::new()
			.merge(figment::providers::Yaml::file(cwd.join("config.yml")))
			.merge(figment::providers::Yaml::file(cwd.join("config.yaml")))
	} else {
		eprintln!(
			"{}warning{}: could not access current working directory; configuration files will be ignored",
			ANSI_YELLOW,
			ANSI_RESET
		);

		Figment::new()
	}
	.merge(figment::providers::Env::prefixed(env_prefix).split("__"))
	.extract()
	.map_err(Box::new)
}

#[test]
fn test_config_order() {
	use dedent::dedent;
	use figment::Jail;
	use serde::Deserialize;

	#[derive(Debug, Clone, Deserialize)]
	struct TestConfig {
		option: String,
	}

	let env_prefix = "FAMEDLY_RUST_UTILS_TEST__";

	Jail::expect_with(|jail| {
		jail.create_file(
			"config.yml",
			dedent!(
				r#"
					option: c
				"#
			),
		)?;

		let cfg: Result<TestConfig, _> = try_parse_config(env_prefix);

		// This *could* be a simple `.expect()`, but it's a nice lil'
		// spot to make stuff panic with error messages if you want to
		// test out what your formatting ends up looking like.
		match cfg {
			Ok(cfg) => assert_eq!(cfg.option, "c"),
			Err(e) => {
				print_parse_config_errors(env_prefix, e);
				panic!("Configuration must be valid")
			}
		};

		jail.create_file(
			"config.yaml",
			dedent!(
				r#"
					option: b
				"#
			),
		)?;

		let cfg: TestConfig = try_parse_config(env_prefix).expect("configuration must be valid");

		assert_eq!(cfg.option, "b");

		jail.set_env("FAMEDLY_RUST_UTILS_TEST__OPTION", "a");

		let cfg: TestConfig = try_parse_config(env_prefix).expect("configuration must be valid");

		assert_eq!(cfg.option, "a");

		Ok(())
	});
}
