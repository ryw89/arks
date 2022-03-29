`arks` is a tool for quickly searching for text within archives.

Basic usage:

`arks some_text some_file.zip`

might yield:

``` bash
your.csv:423:"some text","other","csv","fields"
your.csv:980:"some text","yet","more","data"
```

You can also specify searching only within files in the archive that
match a certain pattern using the `-n, --inner` option. This can
dramatically speed up search time, as files within the archive that
don't match the pattern can be essentially skipped over.

`arks some_text some_file.zip -n csv`

This would give the same output as before, but could be quicker if the
archive contained a lot of other text files without "csv" in their
name.

You can also specify multiple file patterns by delimiting them with the
`|` symbol. For example:

`arks some_text some_file.zip -n "rb|py"`

# Installation

This program is written in Rust, so you'll need the Rust toolchain.
Additionally, some parts of the program rely on the
[`nightly`](https://rust-lang.github.io/rustup/concepts/channels.html)
channel.

Building the program can be performed in the usual way:

```bash
git clone git://github.com/ryw89/arks
cd arks
cargo build --release
```

The binary will end up at `./target/release/arks`.

# Supported archive types

- [x] .tar.bz2
- [x] .tar.gz
- [x] .zip
