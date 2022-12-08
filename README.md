# Stele

Stele is a system for distributing, preserving, and authenticating laws.

## Usage

```
Usage: stele [OPTIONS] <COMMAND>

Commands:
  git   Serve git repositories in the Stele library
  help  Print this message or the help of the given subcommand(s)

Options:
  -l, --library-path <LIBRARY_PATH>  Path to the Stele library. Defaults to cwd [default: .]
  -h, --help                         Print help information
  -V, --version                      Print version information
```

## Development

It is recommended to use the [Rust Analyzer LSP](https://rust-analyzer.github.io/) to get realtime feedback in your editor.

### Testing
  * `cargo test` will run even if there are warnings.
  * `cargo stele test` fails when lints generate warnings. Better for CI.

### Formatting and Linting
  * `cargo stele lint` checks formatting and lints
  * `cargo stele ci` checks everything that Cargo is responsible for on CI
