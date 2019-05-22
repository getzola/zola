+++
title = "GitHub Pages"
weight = 30
+++

By default, GitHub Pages uses Jekyll (A ruby based static site generator),
but you can use whatever you want provided you have an `index.html` file in the root of a branch called `gh-pages`.
That branch name can also be manually changed in the settings of a repository.

We can use any CI server to build and deploy our site. For example:

 * [Github Actions](https://github.com/shalzz/zola-deploy-action)
 * [Travis CI](#travis-ci)

## Travis CI

We are going to use [TravisCI](https://travis-ci.org) to automatically publish the site. If you are not using Travis already,
you will need to login with the GitHub OAuth and activate Travis for the repository.
Don't forget to also check if your repository allows GitHub Pages in its settings.

## Ensure Travis can access your theme

Depending on how you added your theme Travis may not exactly know how to access
it. The best way to ensure it will have full access to the theme is to use git
submodules. When doing this ensure you are using the `https` version of the URL.

```shell
$ git submodule add {THEME_URL} themes/{THEME_NAME}
```

## Allowing Travis to push to GitHub

Before pushing anything, Travis needs a Github private access key in order to make changes to your repository.
If you're already logged in to your account, just click [here](https://github.com/settings/tokens) to go to your tokens page.
Otherwise, navigate to `Settings > Developer Settings > Personal Access Tokens`.
Generate a new token, and give it any description you'd like.
Under the "Select Scopes" section, give it repo permissions. Click "Generate token" to finish up.

Your token will now be visible!
Copy it into your clipboard and head back to Travis.
Once on Travis, click on your project, and navigate to "Settings". Scroll down to "Environment Variables" and input a name of `GH_TOKEN` with a value of your access token.
Make sure "Display value in build log" is off, and then click add. Now Travis has access to your repository.

## Setting up Travis

We're almost done. We just need some scripts in a .travis.yml file to tell Travis what to do.

```yaml
language: minimal

before_script:
  # Download and unzip the zola executable
  # Replace the version numbers in the URL by the version you want to use
  - curl -s -L https://github.com/getzola/zola/releases/download/v0.8.0/zola-v0.8.0-x86_64-unknown-linux-gnu.tar.gz | sudo tar xvzf - -C /usr/local/bin

script:
  - zola build

# If you are using a different folder than `public` for the output directory, you will
# need to change the `zola` command and the `ghp-import` path
after_success: |
  [ $TRAVIS_BRANCH = master ] &&
  [ $TRAVIS_PULL_REQUEST = false ] &&
  zola build &&
  sudo pip install ghp-import &&
  ghp-import -n public &&
  git push -fq https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages
```

If your site is using a custom domain, you will need to mention it in the `ghp-import` command: `ghp-import -c vaporsoft.net -n public`
for example.

Credits: this page is based on the article https://vaporsoft.net/publishing-gutenberg-to-github/
