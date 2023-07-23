+++
title = "Fly.io"
weight = 70
+++

If you don't have an account with fly.io, you can sign up [here](https://fly.io/app/sign-up).

Then install the `flyctl` tool following the instructions [here](https://fly.io/docs/hands-on/install-flyctl/).

Create a `Dockerfile`:

```Dockerfile
FROM ghcr.io/getzola/zola:v0.17.2 AS builder

WORKDIR /app
COPY . .
RUN ["zola", "build"]

FROM joseluisq/static-web-server:2
COPY --from=builder /app/public /public
ENV SERVER_PORT 8080
```

You can now run `fly launch`. It will detect the `Dockerfile` and mostly auto-configure everything. Fill out the necessary information, but say "no" to (1) launching any databases and (2) deploying immediately.

Take note of the hostname assigned to your app.

If you already have a Zola site you must now ensure that `base_url` in `config.toml` is set correctly using the hostname from your app (or whatever domain you have pointing to the app):

    base_url = "https://white-snow-9922.fly.dev"

If you don't have an existing site, initialize one with `zola init -f` and remember to set the correct `base_url`.

You're now ready to launch your site! Run `flyctl deploy` and have fun!

Finally, to set up continuous deployment of your site from GitHub, follow [this](https://fly.io/docs/app-guides/continuous-deployment-with-github-actions/) guide, steps 4-8. Any changes to your site will now be pushed automatically.
