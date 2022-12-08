# Project Notes

## Pedantic Rust Editor Settings
If you would like to make your editor always give you the most verbose feedback in any Rust project, you can use something like this. It can be used as-is in VSCode, or converted to something similar in your editor of choice.

```json
{
    "rust-analyzer.checkOnSave.overrideCommand": [
        "cargo",
        "clippy",
        "--message-format=json",
        "--all-targets",
        "--all-features",
        "--",
        "-W",
        "clippy::all",
        "-W",
        "clippy::pedantic",
        "-W",
        "clippy::restriction",
        "-W",
        "clippy::nursery",
        "-W",
        "clippy::cargo",
        "-W",
        "missing_docs"
    ],
}
```
