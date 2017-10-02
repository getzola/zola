+++
title = "Installation"
weight = 1
+++

Gutenberg provides pre-built binaries for Mac OS, Linux and Windows on the
[Github release page](https://github.com/Keats/gutenberg/releases).

## Using brew on Mac OS

TODO: it's not on brew right now

## Windows

TODO: i have no clue whatsoever about packages in Windows

## Archlinux

TODO: add a `gutenberg-bin` in AUR and explain how to install it

## From source
To build it from source, you will need to have Git, [Rust and Cargo](https://www.rust-lang.org/en-US/)
installed.

From a terminal, you can now run the following command:

```bash
$ cargo build --release
```

The binary will be available in the `target/release` folder.
