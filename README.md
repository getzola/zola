# zola (né Gutenberg)

[![Build Status](https://dev.azure.com/getzola/zola/_apis/build/status/getzola.zola?branchName=master)](https://dev.azure.com/getzola/zola/_build/latest?definitionId=1&branchName=master)
![GitHub all releases](https://img.shields.io/github/downloads/getzola/zola/total)

A fast static site generator in a single binary with everything built-in.

[1 Comand Install](#1-Command-install-build-and-serve-yourself)
[List of features](#list-of-features)
[Where to get zola](#Where-to-get-zola)

Documentation is available on [its site](https://www.getzola.org/documentation/getting-started/installation/) or
in the `docs/content` folder of the repository and the community can use [its forum](https://zola.discourse.group).

This tool and the template engine it is using were born from an intense dislike of the (insane) Golang template engine and therefore of 
Hugo that I was using before for 6+ sites.


# 1 Command install build and serve yourself
### Change "my_site" into whatever you need.

    zola init my_site && cd my_site && zola build && zola serve


# List of features

- Single binary
- Syntax highlighting 
- Sass compilation
- Assets co-location
- (Basic currently) multilingual site suport
- Image processing
- Themes
- Shortcodes
- Internal links
- External link checker
- Table of contents automatic generation
- Automatic header anchors
- Aliases
- Pagination
- Custom taxonomies
- Search with no servers or any third parties involved
- Live reload
- Deploy on many platforms easily: Netlify, Vercel, Cloudflare
- Breadcrumbs


# Where to get zola

| OS | pre-built binaries | Command |
| --- | --- | --- |
| macOS | Brew | brew install zola |
| macOS | MacPorts | sudo port install zola |
| Linux | Arch Linux | pacman -S zola |
| Linux | Alpine Linux | apk add zola |
| Linux | Debian | sudo dpkg -i zola_<version>_amd64_debian_<debian_version>.deb |
| Linux | Fedora | sudo dnf install zola |
| FreeBSD | FreeBSD | pkg install zola |
| OpenBSD | OpenBSD | doas pkg_add zola |
| Snapcraft | Snapcraft | snap install --edge zola |
| Flatpak | Flathub | flatpak install flathub org.getzola.zola |
| Universal | Docker | docker pull ghcr.io/getzola/zola:v0.16.0 |
| Windows | Scoop | scoop install zola |
| Windows | Chocolatey | choco install zola |
