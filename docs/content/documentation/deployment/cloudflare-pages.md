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
7. Toggle _Environment variables_ below and add `ZOLA_VERSION` as _a variable name_. Use `0.13.0` or a different Zola version as the _value_.
8. Finally, save and deploy.

Your website is now built and deployed to Cloudflare's network! You can add a custom domain or modify settings in the Pages dashboard.

You may find documentation and guides like [Getting started with Cloudflare Pages](https://developers.cloudflare.com/pages/getting-started) and
[Deploying Zola with Cloudflare Pages](https://developers.cloudflare.com/pages/how-to/deploy-a-zola-site#deploying-with-cloudflare-pages) in the Developers portal. 
