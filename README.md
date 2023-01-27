# Stelae

Stelae is a system for distributing, preserving, and authenticating laws.

## Contributing

### Setting up environment

1. Install dependencies:
  - Windows:
    - [C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
  - Linux:
    - `sudo apt-get install build-essential pkg-config libssl-dev `
2. Install [Rust](https://www.rust-lang.org/tools/install)
3. Install [Just](https://just.systems/man/en/chapter_3.html) (Our build tool)
4. Windows: install Git Bash, included in [Git for Windows](https://git-scm.com/downloads). 
5. Windows (Optional): install [NuShell](https://www.nushell.sh/book/installation.html) (A fast, cross-platform shell used by Just)
6. We recommend using [VSCode](https://code.visualstudio.com/Download) (default settings provided in repo), but you can use any editor you like.

### Development
- Lints must pass before merging into master
- All code must have tests. Tests should conform to our testing guidelines.
- Run `just` from within the repository to list all available just commands. Currently:
    - `bench`: Run all benchmarks
    - `ci`: Continuous integration - lint, test, benchmark
    - `clippy *FLAGS`: Run clippy maximum strictness. Passes through any flags to clippy.
    - `default`: List all available commands
    - `format`: Format code
    - `lint`: Format code and run strict clippy
    - `test`: Run all tests
- On windows, especially, you may wish to run just through the nu shell, which can be done by calling all commands with the `--shell` command, e.g. `just --shell nu lint`.

## Logging

The ENV variable `RUST_LOG` can be set with one of `trace`, `debug`, `info`, `warn`, `error`. Filters can be set based on the `target` components seen in the logs lines, for example: to use `trace` but turn down the noise from the Actix dispatcher: `RUST_LOG="trace,actix_http::h1::dispatcher=warn"`

See [tracing-subscriber docs](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/fmt/index.html#filtering-events-with-environment-variables) and [env_logger syntax](https://docs.rs/env_logger/latest/env_logger/#enabling-logging]).

## Q&A
- Why do we suggest NuShell?
  - NuShell is almost as fast on windows as cmd, but is compattible with bash. If you do not use NuShell on windows, you will need to make sure Git Bash is installed. If you have performance issues, consider switching to Nu.
