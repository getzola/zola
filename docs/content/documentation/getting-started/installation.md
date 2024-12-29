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

### Gentoo

Zola is available via [GURU](https://wiki.gentoo.org/wiki/Project:GURU).

```sh
$ sudo eselect repository enable guru
$ sudo emaint sync --repo guru
$ sudo emerge --ask www-apps/zola
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

### openSUSE

#### openSUSE Tumbleweed

Zola is [available](https://software.opensuse.org/package/zola?baseproject=ALL) in the official openSUSE Tumbleweed main OSS repository.

```sh
$ sudo zypper install zola
```

#### openSUSE Leap

Zola is [available](https://software.opensuse.org/package/zola?baseproject=ALL) in the official experimental _utilities_ repository.

```sh
$ sudo zypper addrepo https://download.opensuse.org/repositories/utilities/15.6/utilities.repo
$ sudo zypper refresh
$ sudo zypper install zola
```

### pkgsrc

Zola is available in the official package repository, with [pkgin](https://pkgin.net/).

```sh
$ pkgin install zola
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

To avoid having to type this every time, an alias can be created in `~/.bashrc`:

```sh
$ alias zola="flatpak run org.getzola.zola"
```

### NixOS / Nixpkgs

Zola is [available](https://search.nixos.org/packages?show=zola&query=zola)
in the nixpkgs repository. If you're using NixOS, you can install Zola
by adding the following to `/etc/nixos/configuration.nix`:

```
environment.systemPackages = [
  pkgs.zola
];
```

If you're using Nix as a package manager in another OS, you can install it using:

```
nix-env -iA nixpkgs.zola
```

### Via Github Actions

Zola can be installed in a GHA workflow with [taiki-e/install-action](https://github.com/taiki-e/install-action).
Simply add it in your CI config, for example:

```yaml
jobs:
  foo:
    steps:
      - uses: taiki-e/install-action@v2
        with:
          tool: zola@0.19.1
      # ...
```

See the action repo for docs and more examples.

### Docker

Zola is available on [the GitHub registry](https://github.com/getzola/zola/pkgs/container/zola).
It has no `latest` tag, you will need to specify a [specific version to pull](https://github.com/getzola/zola/pkgs/container/zola/versions).

```sh
$ docker pull ghcr.io/getzola/zola:v0.19.1
```

#### Build

```sh
$ docker run -u "$(id -u):$(id -g)" -v $PWD:/app --workdir /app ghcr.io/getzola/zola:v0.19.1 build
```

#### Serve

```sh
$ docker run -u "$(id -u):$(id -g)" -v $PWD:/app --workdir /app -p 8080:8080 ghcr.io/getzola/zola:v0.19.1 serve --interface 0.0.0.0 --port 8080 --base-url localhost
```

You can now browse http://localhost:8080.

> To enable live browser reload, you may have to bind to port 1024. Zola searches for an open
> port between 1024 and 9000 for live reload. The new docker command would be
> `$ docker run -u "$(id -u):$(id -g)" -v $PWD:/app --workdir /app -p 8080:8080 -p 1024:1024 ghcr.io/getzola/zola:v0.19.1 serve --interface 0.0.0.0 --port 8080 --base-url localhost`

#### Multi-stage build

Since there is no shell in the Zola docker image, if you want to use it from inside a Dockerfile, you have to use the
exec form of `RUN`, like:

```Dockerfile
FROM ghcr.io/getzola/zola:v0.19.1 as zola

COPY . /project
WORKDIR /project
RUN ["zola", "build"]
```

## Windows

Zola could be installed using official Winget command:

```sh
$ winget install getzola.zola
```

Also it is available on [Scoop](https://scoop.sh):

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
installed.

From a terminal, you can now run the following commands:

```sh
$ git clone https://github.com/getzola/zola.git
$ cd zola
$ cargo install --path . --locked
$ zola --version
```

If you encountered compilation errors like `error: failed to run custom build command for 'ring v0.16.20'`, you can try the command below instead:

```sh
$ cargo build --release --locked --no-default-features --features=native-tls
```

The binary will be available in the `target/release` directory. You can move it in your `$PATH` to have the
`zola` command available globally:

```sh
$ cp target/release/zola ~/.cargo/bin/zola
```

or in a directory if you want for example to have the binary in the same repository as the site.
