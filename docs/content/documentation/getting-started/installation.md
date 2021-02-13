+++
title = "Installation"
weight = 10
+++

Zola provides pre-built binaries for MacOS, Linux and Windows on the
[GitHub release page](https://github.com/getzola/zola/releases).

### macOS

Zola is available on [Brew](https://brew.sh):

```sh
$ brew install zola
```

### Arch Linux

Zola is available in the official Arch Linux repositories.

```sh
$ pacman -S zola
```

### Alpine Linux

Zola is available in the official Alpine Linux repository, only on the `edge` version for now.

```sh
$ apk add zola --repository http://dl-cdn.alpinelinux.org/alpine/edge/community/
```

### Fedora

Zola has been available in the official repositories since Fedora 29.

```sh
$ sudo dnf install zola
```

### Void Linux

Zola is available in the official Void Linux repositories.

```sh
$ sudo xbps-install zola
```

### FreeBSD

Zola is available in the official package repository.

```sh
$ pkg install zola
```

### OpenBSD

Zola is available in the official package repository.

```sh
$ doas pkg_add zola
```

### Snapcraft

Zola is available on snapcraft:

```sh
$ snap install --edge zola
```

### Docker

Zola is available on [Docker Hub](https://hub.docker.com/r/balthek/zola).
It has no `latest` tag, you will need to specify a [specific version to pull](https://hub.docker.com/r/balthek/zola/tags).

```sh
$ docker pull balthek/zola:0.13.0
$ docker run balthek/zola:0.13.0 --version
```

#### Build

```sh
$ docker run -u "$(id -u):$(id -g)" -v $PWD:/app --workdir /app balthek/zola:0.13.0 build
```

#### Serve

```sh
$ docker run -u "$(id -u):$(id -g)" -v $PWD:/app --workdir /app -p 8080:8080 balthek/zola:0.13.0 serve --interface 0.0.0.0 --port 8080 --base-url localhost
```

You can now browse http://localhost:8080.

> To enable live browser reload, you may have to bind to port 1024. Zola searches for an open
> port between 1024 and 9000 for live reload. The new docker command would be
> `$ docker run -u "$(id -u):$(id -g)" -v $PWD:/app --workdir /app -p 8080:8080 -p 1024:1024 balthek/zola:0.13.0 serve --interface 0.0.0.0 --port 8080 --base-url localhost`

## Windows

Zola is available on [Scoop](https://scoop.sh):

```sh
$ scoop install zola
```

and [Chocolatey](https://chocolatey.org/):

```sh
$ choco install zola
```

Zola does not work in PowerShell ISE.

## From source

To build Zola from source, you will need to have Git, [Rust (at least 1.45) and Cargo](https://www.rust-lang.org/)
installed. You will also need to meet additional dependencies to compile [libsass](https://github.com/sass/libsass):

- OSX, Linux and other Unix-like operating systems: `make` (`gmake` on BSDs), `g++`, `libssl-dev`
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
  - Then, invoke `nix-shell`. This opens a shell with the above dependencies. Then, run `cargo build --release` to build the project.
- Windows (a bit trickier): updated `MSVC` and overall updated VS installation

From a terminal, you can now run the following command:

```sh
$ cargo build --release
```

The binary will be available in the `target/release` directory. You can move it in your `$PATH` to have the
`zola` command available globally or in a directory if you want for example to have the binary in the
same repository as the site.
