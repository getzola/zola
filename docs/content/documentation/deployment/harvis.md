+++
title = "harvis"
weight = 70
+++

[harvis](https://harvis.dev) is zero-setup static hosting: you upload the files in your `public`
directory as-is and get a live URL — no account, no configuration, no build pipeline on their side.

## Deploying

Build your site, then run the harvis CLI in the output directory:

```bash
zola build
npx harvis public
```

The command uploads the folder and prints the URL your site is live at, together with a private
claim link you can use to attach the site to an account later. Running the same command again
updates the site in place.

The free tier covers small sites; see [harvis.dev](https://harvis.dev) for limits and pricing.
