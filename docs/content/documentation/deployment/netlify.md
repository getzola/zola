+++
title = "Netlify"
weight = 20
+++

Netlify provides best practices like SSL, CDN distribution, caching and continuous deployment 
with no effort. This very site is hosted by Netlify and automatically deployed on commits.

If you don't have an account with Netlify, you can [sign up](https://app.netlify.com) for one.

Once you are in the admin interface, you can add a site from a Git provider (GitHub, GitLab or Bitbucket). At the end
 of this process, you can select the deploy settings for the project:
 
 - build command: `GUTENBERG_VERSION=0.3.1 gutenberg build` (replace the version number in the variable by the version you want to use)
 - publish directory: the path to where the `public` directory is
 
With this setup, your site should be automatically deployed on every commit on master.

However, if you want to use everything that Netlify gives you, you should also publish temporary sites for pull requests.

This is done by adding the following `netlify.toml` file in your repository and removing the build command/publish directory in
the admin interface.

```toml
[build]
# assuming the gutenberg site is in a docs folder, if it isn't you don't need 
# to have a `base` variable but you do need the `publish` and `command`
base    = "docs"
publish = "docs/public"
command = "gutenberg build"

[build.environment]
# Set the version name that you want to use and Netlify will automatically use it
GUTENBERG_VERSION = "0.3.1"

# The magic for deploying previews of branches
# We need to override the base url with what the url of the preview is ($DEPLOY_PRIME_URL)
# otherwise links would not work properly
[context.deploy-preview]
command = "gutenberg build --base-url $DEPLOY_PRIME_URL"

```


