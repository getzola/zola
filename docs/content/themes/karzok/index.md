
+++
title = "karzok"
description = "The theme for launching fast documentation sites"
template = "theme.html"
date = 2022-10-04T04:04:47-05:00

[extra]
created = 2022-10-04T04:04:47-05:00
updated = 2022-10-04T04:04:47-05:00
repository = "https://github.com/kogeletey/karzok.git"
homepage = "https://github.com/kogeletey/karzok"
minimum_version = "0.15.0"
license = "MIT"
demo = "https://karzok.re128.org"

[extra.author]
name = "Konrad Geletey"
homepage = ""
+++        

<p align="center">
  <a href="https://github.com/kogeletey/karzok/actions"><img src="https://flat.badgen.net/github/checks/kogeletey/karzok"  alt="github workflows status" /></a>
  <a href="https://github.com/kogeletey/karzok/blob/develop/LICENSE"><img src="https://flat.badgen.net/github/license/kogeletey/karzok" alt="license a repository" /></a>
  <a href="https://github.com/kogeletey/karzok/releases"><img src="https://flat.badgen.net/github/release/kogeletey/karzok" alt="latest release as a repository" /></a>
  <a href="https://framagit.org/kogeletey/nebra"><img alt="pipeline status re128" src="https://framagit.org/kogeletey/nebra/badges/develop/pipeline.svg" /></a>
</p>

# Karzok

A theme for your documentation. Fast and secure

![screenshot](./screenshot.png)

## Demo

[Karzok](https://karzok.re128.org)

## Requirements

Karzok uses npm,zola to dependency managment,rendering, scripts and plugins.

### Install

1. [Zola](https://www.getzola.org/documentation/getting-started/installation/)
2. [Node.js](https://nodejs.org/)
3. [rsync](https://rsync.samba.org)

for your platform.

### Optional

- [docker](https://docs.docker.com/engine/install/)
   > for packaging container and production

## Get Started

### 1. Create a new zola site

```zsh
zola init zola_site
```

### 2. Download this theme to you themes directory:

```zsh
git clone https://git.sr.ht/~kogeletey/karzok zola_site/themes
```

or install as submodule:

```zsh
cd zola_site
git init # if your project is a git repository already, ignore this command
git submodule add https://git.sr.ht/~kogeletey/karzok zola_site/themes
```

### 3. Configuration. Open in favorite editor `config.toml`

```toml
base_url = "https://karzok.example.net" # set-up for production
theme = "karzok"
```

See more in [configuration](https://karzok.re128.org/configure/)

### 4. Added new content

```zsh
    cp ./themes/content/_index.md content/_index.md
    # a template will appear with which you can quickly start writing
    # cp ./themes/content/tmpl.md content/filename.md
```

how you can give freedom to your creativity

### 5. Run the project

i. development enviroment

1. Install node dependencies needed to work

```zsh
npm ci
npm run gen 
```

2. Just run `zola serve` in the root path of the project

```zsh
zola serve
```

Open in favorite browser [http://127.0.0.1:1111](http://127.0.0.1:1111). Saved
changes live reolad.

ii. production enviroment

- with docker

1. Write file for container

```Dockerfile
FROM ghcr.io/kogeletey/karzok:latest AS build-stage
# or your path to image
ADD . /www
WORKDIR /www
RUN sh /www/build.sh 

FROM nginx:stable-alpine

COPY --from=build-stage /www/public /usr/share/nginx/html

EXPOSE 80
```

2.  Run the your container
```zsh
docker build -t <your_name_image> . &&\
docker run -d -p 8080:8080 <your_name_image> 
```
- using gitlab-ci and gitlab-pages

```yml
image: ghcr.io/kogeletey/karzok:latest # or change use your registry

pages: 
  script:
    - sh /www/build.sh   
    - mv /www/public public
  artifacts:
    paths:
      - public/
```

Open in favorite browser [https://localhost:8080](http://localhost:8080)

## License

This program is Free Software: You can use, study share and improve it at your
will. Specifically you can redistribute and/or modify it under the terms of the
[MIT](https://mit-license.org/)

# Contribute

Make sure to read the [Code of Conduct](https://karzok.re128.org/reference/code-of-conduct/)

## Find bugs and come up with features

On the [todo.sr.ht](https://todo.sr.ht/~kogeletey/karzok) or
[github issues](https://github.com/kogeletey/karzok/issues)

## Improve Code

The karzok is stored in the repository at
[sr.ht](https://sr.ht/~kogeletey/karzok) and mirror
[github](https://github.com/kogeletey/karzok)

> Thank you so much for any help

        