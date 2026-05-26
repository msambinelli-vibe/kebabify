# kebabify

`kebabify` is a Rust CLI to rename files and directories to a consistent naming pattern:

- `kebab-case` (default)
- `snake_case`

It is designed for safe batch renaming, including dry-run mode, conflict handling, and Unicode controls.

## Features

- Renames individual files.
- Processes directories (current level or recursively).
- Can rename nested directories (`--dirs`).
- Can rename the input directory itself (`--rename-root`).
- Simulation mode without changing files (`--dry-run`).
- Conflict strategies: `suffix`, `skip`, `error`.
- Unicode modes: `ascii` (remove accents) or `preserve`.
- Hidden files are ignored by default (include with `--all`).
- Symlinks are not followed by default (enable with `--follow-symlinks`).

## Installation

### Requirements

- Rust (stable toolchain)
- Cargo

### Local build

```bash
cargo build --release
```

Binary output:

```bash
target/release/kebabify
```

### Run without installing

```bash
cargo run -- "My File.pdf"
```

## Usage

```bash
kebabify [OPTIONS] <PATH>...
```

### Arguments

- `<PATH>...`: files or directories to process.

### Options

- `-n, --dry-run`
  - Show planned renames without applying changes.

- `-r, --recursive`
  - Process directories recursively.

- `-d, --dirs`
  - Also rename nested directories inside each input directory.

- `--rename-root`
  - Rename the input directory itself.

- `-s, --style <STYLE>`
  - Naming style: `kebab` or `snake`.
  - Default: `kebab`.

- `-a, --all`
  - Include hidden files and directories.

- `--follow-symlinks`
  - Follow symbolic links.

- `--on-conflict <MODE>`
  - Conflict strategy: `suffix`, `skip`, `error`.
  - Default: `suffix`.

- `--unicode <MODE>`
  - Unicode handling: `ascii` or `preserve`.
  - Default: `ascii`.

## Processing semantics

- If `PATH` is a **file**: only that file is renamed.
- If `PATH` is a **directory** without `--recursive`: only immediate files are renamed.
- If `PATH` is a **directory** with `--recursive`: files in the full tree are renamed.
- If `--dirs` is enabled: nested directories are renamed too.
- If `--rename-root` is enabled: the `PATH` directory itself is renamed.

## Examples

Rename a single file:

```bash
kebabify "My File.pdf"
```

Process a directory (current level only):

```bash
kebabify ~/Downloads
```

Process multiple paths:

```bash
kebabify ~/Downloads ~/Desktop/file.txt
```

Preview recursive renaming including directories:

```bash
kebabify ~/papers -r --dirs --dry-run
```

Use `snake_case`:

```bash
kebabify -s snake ~/Downloads
```

Fail on conflicts:

```bash
kebabify --on-conflict error ~/Downloads
```

Preserve Unicode:

```bash
kebabify --unicode preserve "Olá Mundo.txt"
```

## Output

The CLI prints each rename as:

```text
source -> target
```

Then prints a summary to `stderr`:

```text
planned: X, renamed: Y
```

In `--dry-run` mode:

```text
planned: X, renamed: Y (dry-run)
```

## Development

Run tests:

```bash
cargo test
```

Run lint (optional):

```bash
cargo clippy --all-targets --all-features -- -D warnings
```

Format code (optional):

```bash
cargo fmt
```

## Packaging

### Arch Linux

This repository includes a `PKGBUILD`.

Typical flow:

```bash
makepkg -si
```

> Update fields like `url` and `Maintainer` in `PKGBUILD` for your real repository/release setup.

### Automated releases

Included GitHub Actions workflows:

- Test CI on pushes to `master` and pull requests.
- Release on `v*` tags, with artifacts:
  - Linux binary tarball
  - `.deb` package (amd64)
  - Arch bundle (PKGBUILD)

## License

MIT.
