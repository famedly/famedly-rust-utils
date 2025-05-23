<!--
SPDX-FileCopyrightText: 2025 Famedly GmbH (info@famedly.com)

SPDX-License-Identifier: Apache-2.0
-->

# Famedly rust utils

[![rust workflow status][badge-rust-workflow-img]][badge-rust-workflow-url]

[badge-rust-workflow-img]: https://github.com/famedly/famedly-rust-utils/actions/workflows/rust.yml/badge.svg
[badge-rust-workflow-url]: https://github.com/famedly/famedly-rust-utils/commits/main

Various rust utility functions and types originally developed for internal use at Famedly. For docs see [`docs.rs/famedly_rust_utils`](https://docs.rs/famedly_rust_utils/latest/famedly_rust_utils/).

## Lints

```sh
cargo clippy --workspace --all-targets
```

and this in your IDE:
```sh
cargo clippy --workspace --all-targets --message-format=json
```

## Pre-commit usage

1. If not installed, install with your package manager, or `pip install --user pre-commit`
2. Run `pre-commit autoupdate` to update the pre-commit config to use the newest template
3. Run `pre-commit install` to install the pre-commit hooks to your local environment

---

# Famedly

**This project is part of the source code of Famedly.**

We think that software for healthcare should be open source, so we publish most
parts of our source code at [github.com/famedly](https://github.com/famedly).

Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on our code of
conduct, and the process for submitting pull requests to us.

For licensing information of this project, have a look at the [LICENSE](LICENSE.md)
file within the repository.

If you compile the open source software that we make available to develop your
own mobile, desktop or embeddable application, and cause that application to
connect to our servers for any purposes, you have to agree to our Terms of
Service. In short, if you choose to connect to our servers, certain restrictions
apply as follows:

- You agree not to change the way the open source software connects and
  interacts with our servers
- You agree not to weaken any of the security features of the open source software
- You agree not to use the open source software to gather data
- You agree not to use our servers to store data for purposes other than
  the intended and original functionality of the Software
- You acknowledge that you are solely responsible for any and all updates to
  your software

No license is granted to the Famedly trademark and its associated logos, all of
which will continue to be owned exclusively by Famedly GmbH. Any use of the
Famedly trademark and/or its associated logos is expressly prohibited without
the express prior written consent of Famedly GmbH.

For more
information take a look at [Famedly.com](https://famedly.com) or contact
us by [info@famedly.com](mailto:info@famedly.com?subject=[GitLab]%20More%20Information%20)
