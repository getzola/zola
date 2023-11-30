+++
title = "Cloudflare Pages"
weight = 60
+++

Cloudflare is a cloud solutions provider with a huge proprietary content delivery network (CDN). Like Netlify or Vercel, Cloudflare Pages makes the deployment process flexible and easy. You can add a GitHub repo to the service and build & host Zola-based websites after each PR automatically.

## Step-by-step

1. Sign in or create a new Cloudflare account and choose _"Pages"_ on the right nav column
2. Press the button _"Create a project"_
3. Select the GitHub repo that contains your Zola website and connect it to Cloudflare Pages
4. Click _"Begin setup"_
5. Enter your project name. Keep in mind that if you would like to use the default Pages domain (pages.dev), this will be your website's future URL ("yourprojectname.pages.dev"). Additionally, select a production branch.
6. In _Build settings_, select Zola as the _Framework preset_. _Build command_ and _Build output directory_ will be filled automatically.
7. Toggle _Environment variables_ below and add `ZOLA_VERSION` as _a variable name_. Use `0.17.2` or a different Zola version as the _value_.
8. Finally, save and deploy.

Your website is now built and deployed to Cloudflare's network! You can add a custom domain or modify settings in the Pages dashboard.

You may find documentation and guides like [Getting started with Cloudflare Pages](https://developers.cloudflare.com/pages/getting-started) and
[Deploying Zola with Cloudflare Pages](https://developers.cloudflare.com/pages/how-to/deploy-a-zola-site#deploying-with-cloudflare-pages) in the Developers portal.

## Troubleshooting

Some tips to help troubleshoot issues getting started with Cloudflare Pages.

### `zola: not found`

If you see build output that resembles something like this:

```sh
23:03:54.609	> build
23:03:54.609	> zola build $BUILD_OPTS && npx tailwindcss -i ./public/input.css -o ./public/style.css -m
23:03:54.609
23:03:54.621	sh: 1: zola: not found
23:03:54.635	Failed: Error while executing user command. Exited with error code: 127
23:03:54.644	Failed: build command exited with code: 1
23:03:55.699	Failed: error occurred while running build command
```

Then it might be due to an [outstanding issue](https://github.com/cloudflare/pages-build-image/issues/3#issuecomment-1646873666). There are currently two recommended workarounds:

#### Change the **build system version** to `v1`

From within the workers & pages dash, go to the following:
<Your Project> Settings > Builds & deployments > Build system version > Configure build system

Then select `v1` and save.

#### Or use `UNSTABLE_PRE_BUILD` environment variable + `asdf`

From within the workers & pages dash, do the following:
<Your Project> Settings > Environment variables > Edit variables

And add an environment variable `UNSTABLE_PRE_BUILD`, with the following value and save.

```sh
asdf plugin add zola https://github.com/salasrod/asdf-zola && asdf install zola 0.17.2 && asdf global zola 0.17.2
```




