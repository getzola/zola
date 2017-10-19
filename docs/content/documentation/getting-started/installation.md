+++
title = "Installation"
weight = 1
+++

Gutenberg provides pre-built binaries for MacOS, Linux and Windows on the
[GitHub release page](https://github.com/Keats/gutenberg/releases).

## Mac OS

Gutenberg is not currently available on Homebrew at the moment.

If you can help package it, please comment on https://github.com/Keats/gutenberg/issues/12
if you encounter any issues.

## Windows

I am not aware of the package management state in Windows.

If you can help package it, please comment on https://github.com/Keats/gutenberg/issues/12
if you encounter any issues.

## Archlinux

Use your favourite AUR helper to install the `gutenberg-bin` package.

```bash
$ yaourt -S gutenberg-bin
```

## From source
To build it from source, you will need to have Git, [Rust and Cargo](https://www.rust-lang.org/en-US/)
installed.

From a terminal, you can now run the following command:

```bash
$ cargo build --release
```

The binary will be available in the `target/release` folder.
