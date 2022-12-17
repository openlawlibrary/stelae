# Stele

Stele is a system for distributing, preserving, and authenticating laws.

## Contributing

### Setting up environment

1. Install dependencies:
  - Windows:
    - [C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/)
  - Linux:
    - ...
2. Install [Rust](https://www.rust-lang.org/tools/install)
3. Install [Just](https://just.systems/man/en/chapter_3.html) (Our build tool)
4. Install [NuShell](https://www.nushell.sh/book/installation.html) (A fast, cross-platform shell used by Just)
5. We recommend using [VSCode](https://code.visualstudio.com/Download) (default settings provided in repo)

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



## Q&A
- Why are we using NuShell?
  - Build needs to work across both windows and linux, so we must have a shell that works on both. Because speed is important for linting it must also be fast on both. In tests on Windows, Nu was approximately 10x faster to start up than PowerShell, and 2x faster than Git for windows' Bash.
