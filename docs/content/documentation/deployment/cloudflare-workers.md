+++
title = "Cloudflare Workers"
weight = 60
+++

Cloudflare is a cloud solutions provider with a huge proprietary content delivery network (CDN). Like Netlify or Vercel, Cloudflare Workers makes the deployment process flexible and easy. You can add a GitHub repo to the service and build & host Zola-based websites after each PR automatically.

## Prepare your repo

Before creating a Cloudflare Worker you have to add a configuration for [Wrangler](https://developers.cloudflare.com/workers/wrangler/) to build your site when the default command `npx wrangler deploy` is called.

### Create a build script

First, you'll need a build script that fetches and extracts Zola from GitHub Releases. If your site's repository has submodules, e.g. a theme, performing a submodule update is also necessary since Cloudflare doesn't clone the repo recursively. Let's call the script `build.sh` and add it at the root of the repo.

```bash
#!/usr/bin/env bash
main() {
    ZOLA_VERSION=0.22.1

    curl -sLJO "https://github.com/getzola/zola/releases/download/v${ZOLA_VERSION}/zola-v${ZOLA_VERSION}-x86_64-unknown-linux-gnu.tar.gz"
    tar -xf zola-v${ZOLA_VERSION}-x86_64-unknown-linux-gnu.tar.gz

    git submodule update --init --recursive

    ./zola build
}

set -euo pipe
```

### Add the wrangler configuration

Second, a `wrangler.toml` (also at the project's root) is used to point Wrangler to the build script and the target directory of the generated site. `name` and `compatibility_date` are required by Wrangler.[^1] Just use your site's name and the current date.

```toml
name = "blog"
compatibility_date = "2026-01-22"

[build]
command = "./build.sh"

[assets]
directory = "./public"
```

## Create a Worker

1. Sign in or create a new Cloudflare account and choose _"Workers and Pages"_ on the nav column
2. Press the button _"Create a project"_
3. Select the GitHub or GitLab repo that contains your Zola website and connect it to Cloudflare Workers
4. Keep the defaults and hit _"Deploy"_

Your website is now built and deployed to Cloudflare's network! You can add a custom domain or modify settings in the Workers dashboard.

You may find more documentation at [Creating a Workers application using the Cloudflare dashboard](https://developers.cloudflare.com/workers/get-started/dashboard/).

[^1]: https://developers.cloudflare.com/workers/wrangler/configuration/#inheritable-keys
