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

Zola is also available on [MacPorts](https://www.macports.org):

```sh
$ sudo port install zola
```

### Arch Linux

Zola is available in the official Arch Linux repositories.

```sh
$ pacman -S zola
```

### Alpine Linux

Zola is available in the official Alpine Linux community repository since Alpine v3.13.

See this section of the Alpine Wiki explaining how to enable the community repository if necessary: https://wiki.alpinelinux.org/wiki/Repositories#Enabling_the_community_repository

```sh
$ apk add zola
```

### Debian

Zola is available over at [barnumbirr/zola-debian](https://github.com/barnumbirr/zola-debian).
Grab the latest `.deb` for your Debian version then simply run:

```sh
$ sudo dpkg -i zola_<version>_amd64_debian_<debian_version>.deb
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

### Flatpak

Zola is available as a flatpak on [flathub](https://flathub.org):

```sh
$ flatpak install flathub org.getzola.zola
```

To use zola:

```sh
$ flatpak run org.getzola.zola [command]
```

To avoid having to type this everytime, an alias can be created in `~/.bashrc`:

```sh
$ alias zola="flatpak run org.getzola.zola"
```

### Docker

Zola is available on [the GitHub registry](https://github.com/getzola/zola/pkgs/container/zola).
It has no `latest` tag, you will need to specify a [specific version to pull](https://github.com/getzola/zola/pkgs/container/zola/versions).

```sh
$ docker pull ghcr.io/getzola/zola:v0.16.0
```

#### Build

```sh
$ docker run -u "$(id -u):$(id -g)" -v $PWD:/app --workdir /app ghcr.io/getzola/zola:v0.16.0 build
```

#### Serve

```sh
$ docker run -u "$(id -u):$(id -g)" -v $PWD:/app --workdir /app -p 8080:8080 ghcr.io/getzola/zola:v0.16.0 serve --interface 0.0.0.0 --port 8080 --base-url localhost
```

You can now browse http://localhost:8080.

> To enable live browser reload, you may have to bind to port 1024. Zola searches for an open
> port between 1024 and 9000 for live reload. The new docker command would be
> `$ docker run -u "$(id -u):$(id -g)" -v $PWD:/app --workdir /app -p 8080:8080 -p 1024:1024 ghcr.io/getzola/zola:v0.16.0 serve --interface 0.0.0.0 --port 8080 --base-url localhost`


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
To build Zola from source, you will need to have Git, [Rust and Cargo](https://www.rust-lang.org/)
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

If you encountered compilation errors like `error: failed to run custom build command for 'ring v0.16.20'`, you can try the command below instead:

```sh
$ cargo build --release --no-default-features --features=native-tls
```

The binary will be available in the `target/release` directory. You can move it in your `$PATH` to have the
`zola` command available globally or in a directory if you want for example to have the binary in the
same repository as the site.
