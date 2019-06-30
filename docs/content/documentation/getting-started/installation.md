+++
title = "Installation"
weight = 1
+++

Zola provides pre-built binaries for MacOS, Linux and Windows on the
[GitHub release page](https://github.com/getzola/zola/releases).

### macOS

Zola is available on [Brew](https://brew.sh):

```bash
$ brew install zola
```

### Arch Linux

Use your favourite AUR helper to install the `zola-bin` package.

```bash
$ yay -S zola-bin
```

### Fedora

Zola is available in official repositories since Fedora 29.

```sh
$ sudo dnf install zola
```

### Snapcraft

Zola is available on snapcraft:

```bash
$ snap install --edge zola
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

Zola does not work in PowerShell ISE.

## From source
To build it from source, you will need to have Git, [Rust (at least 1.31) and Cargo](https://www.rust-lang.org/)
installed. You will also need additional dependencies to compile [libsass](https://github.com/sass/libsass):

- OSX, Linux and other Unix: `make` (`gmake` on BSDs), `g++`, `libssl-dev`
  - NixOS: Create a `shell.nix` file in the root of the cloned project with the following contents:
  ```nix
   with import <nixpkgs> {};

   pkgs.mkShell {
     buildInputs = [
       libsass
       openssl
       pkgconfig
    ];
   }
  ```
  - Then invoke `nix-shell`. This opens a shell with the above dependencies. You then run `cargo build --release` to build the project.
- Windows (a bit trickier): updated `MSVC` and overall updated VS installation

From a terminal, you can now run the following command:

```bash
$ cargo build --release
```

The binary will be available in the `target/release` folder. You can move it in your `$PATH` to have the
`zola` command available globally or in a directory if you want for example to have the binary in the
same repository as the site.
