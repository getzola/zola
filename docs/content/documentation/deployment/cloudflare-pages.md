+++
title = "Cloudflare Pages"
weight = 60
+++

Cloudflare is a cloud solutions provider with a huge proprietary content delivery network. Like Netlify or Vercel Cloudflare Pages makes deployment process flexible and easy. You can add GitHub repo to the service and build&host Zola-bases websites after each PR automatically. 

## Step-by-step

1. Open or create the new one Cloudflare account and choice Pages on the right nav column
2. Press the button _"Create a project"_
3. Select the GitHub repo that contains your Zola website and connect it to Cloudflare Pages
4. Click _"Begin setup"_
5. Enter your project name and keep in mind, if you would like to use default Pages domain (pages.dev) it will be your website future URL, like `yourprojectname.pages.dev`. Also, select a production branch.
6. In _Build settings_ select Zola as a _Framework preset_. _Build command_ and _Build output directory_ will be filled automatically. 
7. Toggle _Environment variables_ below and add `ZOLA_VERSION` as _a variable name_ and `0.13.0` or an actual Zola's version as a _value_.
8. Save and deploy.

Your website is now built and deployed to Pages. You may add custom domain or change some settings in the Pages dashboard.

Also, you may find well documented settings and howto [Getting started with Cloudflare Pages](https://developers.cloudflare.com/pages/getting-started) and
[Deloying Zola with Cloudflare Pages](https://developers.cloudflare.com/pages/how-to/deploy-a-zola-site#deploying-with-cloudflare-pages) in Developers portal. 