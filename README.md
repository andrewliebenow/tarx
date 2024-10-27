# `tarx`

`tarx` is similar to `ouch` (https://github.com/ouch-org/ouch). Points of differentiation:

- 👍🏻 `tarx` and its dependencies are written in memory-safe languages (Rust and Go).
  - `ouch` binds to C libraries.
- 👍🏻 `tarx` and its dependencies are permissively licensed.
  - `ouch` uses the `unrar` crate, which uses the proprietary `UnRAR` library.
- 👍🏻 `tarx` can extract many encrypted archives (.7z, .rar, and .zip). Note that not all encryption schemes supported by these formats may work.
  - `ouch` does not support extracting any encrypted archives.
- ❓ `tarx` always extracts the contents of the archive to a directory with the name of archive, excluding the file extension.
  - `ouch` has a `smart_unpack` function (https://github.com/ouch-org/ouch/blob/4ac8e2ba9126e50af73b12cdfd9955a3161f2bab/src/commands/decompress.rs#L233-L239) that causes its behavior to vary depending on whether the archive has one or multiple root entries (directories or files). This "smart" functionality cannot be disabled, and annoyed me so much that I wrote `tarx`. You probably don't care about this.
- 👎🏻 `tarx` is untested ("it works on my machine").
  - `ouch` has a test suite and thousands of users.
- 👎🏻 `tarx` only supports decompression, and only supports archive files (e.g. `directory-to-archive.tar.gz`).
  - `ouch` supports compression, and can decompress single files (e.g. `downloaded-wikipedia-article.html.gz`).
- 👎🏻 The decompression done via FFI to Go code is very naive, and requires the entire archive, plus its decompressed contents, to fit in memory.

## Installation

```Shell
# TODO Publish to crates.io
cargo install --git https://github.com/andrewliebenow/tarx
```

Decompression of `.rar`, `.tar.bz2`, and `.tar.zst` files is provided via FFI to Go code. This requires the `foreign` feature to be enabled (which it is by default). The Go FFI will not work with musl until https://github.com/golang/go/issues/13492 is resolved. In musl environments, disable the `foreign` feature with `--no-default-features`:

```Shell
cargo install --git https://github.com/andrewliebenow/tarx --no-default-features
```

By default, `tarx` uses the allocator provided by the `dlmalloc` crate instead of the system allocator. This too can be disabled with `--no-default-features`.

## Usage

```
❯ tarx --help
Extract a .7z, .rar, .tar, .tar.bz2, .tar.gz, .tar.xz, .tar.zst, or .zip file to a new directory

Usage: tarx [OPTIONS] <ARCHIVE_FILE_PATH>

Arguments:
  <ARCHIVE_FILE_PATH>  Path of the archive file to be processed

Options:
  -p, --password <PASSWORD>  Password of the encrypted archive file to be processed
  -t, --type-password        Interactively enter the password of the encrypted archive file
  -l, --list-files           List files instead of extracting them (not currently implemented for .7z and .zip files)
  -h, --help                 Print help
  -V, --version              Print version
```

## License

Author: Andrew Liebenow

Licensed under the MIT License, see <a href="./LICENSE">./LICENSE</a>.

`tarx` depends on libraries written by other authors. See <a href="./Cargo.toml">./Cargo.toml</a> and <a href="./foreign/go.mod">./foreign/go.mod</a> for its direct (i.e. non-transitive) dependencies.

Note that all dependencies of `tarx` (direct _and_ transitive) are permissively licensed (not copyleft).
