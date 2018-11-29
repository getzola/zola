+++
title = "Installation"
weight = 1
+++

Zola provides pre-built binaries for MacOS, Linux and Windows on the
[GitHub release page](https://github.com/getzola/zola/releases).

## Linux

### Arch Linux

Use your favourite AUR helper to install the `zola-bin` package.

```bash
$ yay -S zola-bin
```

### Snapcraft

Zola is available on snapcraft:

```bash
$ snap install --edge --classic zola
```

## Windows

Zola is available on [Scoop](http://scoop.sh):

```bash
$ scoop install zola
```

And [Chocolatey](https://chocolatey.org/):

```bash
$ choco install zola
```

## From source
To build it from source, you will need to have Git, [Rust (at least 1.28) and Cargo](https://www.rust-lang.org/)
installed. You will also need additional dependencies to compile [libsass](https://github.com/sass/libsass):

- OSX, Linux and other Unix: `make` (`gmake` on BSDs), `g++`, `libssl-dev`
- Windows (a bit trickier): updated `MSVC` and overall updated VS installation

From a terminal, you can now run the following command:

```bash
$ cargo build --release
```

The binary will be available in the `target/release` folder. You can move it in your `$PATH` to have the
`zola` command available globally or in a directory if you want for example to have the binary in the
same repository as the site.


## Older versions (Gutenberg)
Before 0.5, Zola was called Gutenberg. Since the process of updating package managers to Zola is on-going, you can still
download previous versions of Gutenberg on your package manager of choice in the meantime.

### macOS

Gutenberg is available on [Brew](https://brew.sh):

```bash
$ brew install gutenberg
```

### Windows

Gutenberg is available on [Scoop](http://scoop.sh):

```bash
$ scoop install gutenberg
```

And [Chocolatey](https://chocolatey.org/):

```bash
$ choco install gutenberg
```

### Arch Linux

Use your favourite AUR helper to install the `gutenberg-bin` package.

```bash
$ yaourt -S gutenberg-bin
```

### Void Linux

From the terminal, run the following command:

```bash
$ xbps-install -S gutenberg
```
