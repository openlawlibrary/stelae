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

Lint: `cargo stele lint`
Test: `cargo stele test`
