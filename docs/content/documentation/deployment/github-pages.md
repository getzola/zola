+++
title = "GitHub Pages"
weight = 30
+++

You have 2 options to deploy to GitHub Pages:

- use Pages Artifacts and handle publishing via GitHub Actions
- use a different branch (eg `gh-pages`) where you commit the generated files

If you need to use your own custom domain, follow the GitHub docs from <https://docs.github.com/en/pages/configuring-a-custom-domain-for-your-github-pages-site/about-custom-domains-and-github-pages>

## Pages Artifacts

There is an official action from Zola: <https://github.com/getzola/github-pages>

Follow its README (don't forget to enable the GitHub Action source in the settings) and you should get
your site up and running via the CI.

## Branch publishing

There is another action for that approach: <https://github.com/shalzz/zola-deploy-action>

