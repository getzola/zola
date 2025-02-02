+++
title = "Codeberg Pages"
weight = 50
+++

We are going to use the Woodpecker CI hosted by Codeberg to host the site on Codeberg Pages.

## Configuring your repository

It is required that you create a repository on Codeberg that contains only your Zola project. The [Zola directory structure](https://www.getzola.org/documentation/getting-started/directory-structure/) should be in the root of your repository.

Information on how to create and manage a repository on Codeberg can be found at <https://docs.codeberg.org/getting-started/first-repository/>.

## Ensuring that Woodpecker CI can access your theme

Depending on how you added your theme, your repository may not contain it. The best way to ensure that the theme is added is to use submodules. Make sure you use the `https` version of the URL.

```bash
git submodule add <theme_url> themes/<theme_name>
```

For example, this could look like:

```bash
git submodule add https://github.com/getzola/hyde.git themes/hyde
```

## Setting up Woodpecker CI

Assuming you have access to [Codeberg's Woodpecker CI](https://docs.codeberg.org/ci/), we can build the site and automatically deploy it to [Codeberg Pages](https://codeberg.page) on every commit.

First, place the following sample [Zola CI file](https://codeberg.org/Codeberg-CI/examples/src/branch/main/Zola/.woodpecker.yaml) in the root of your project:

```yaml
# Exclude the pipeline to run on the pages branch
when:
  branch:
    exclude: pages

# Clone recursively to fully clone the themes given as Git submodules
clone:
  git:
    image: woodpeckerci/plugin-git
    settings:
      recursive: true

steps:
  # Build Zola static files
  build:
    image: alpine:edge
    commands:
      - apk add zola
      - zola build
    when:
      event: [push, pull_request]

  publish:
    image: bitnami/git
    environment:
      CBMAIL:
        from_secret: "mail"
      CBTOKEN:
        from_secret: "codeberg_token"
    commands:
      # Configure Git
      - git config --global user.email "$${CBMAIL}"
      - git config --global user.name "Woodpecker CI"
      # Clone the output branch
      - git clone --branch pages https://$${CBTOKEN}@codeberg.org/$CI_REPO.git $CI_REPO_NAME
      # Enter the output branch
      - cd $CI_REPO_NAME
      # Remove old files
      - git rm -r "*" || true # Don't fail if there's nothing to remove
      # Copy the output of the build step
      - cp -ar ../public/. .
      # Commit and push all static files with the source commit hash
      - git add --all
      - git commit -m "Woodpecker CI ${CI_COMMIT_SHA} [SKIP CI]" --allow-empty
      - git push
    when:
      event: [push]
```

Then add the following secrets to [Woodpecker](https://ci.codeberg.org/):

- `mail`: Your email address as used by Codeberg.
- `codeberg_token`: [Codeberg access token](https://docs.codeberg.org/advanced/access-token/) with `write:repository` permission.

Once done, you can trigger the CI by pushing something to the repository, and Woodpecker will build the site and copy the resulting site to the `pages` branch and it will be available at `https://<user>.codeberg.page/<repository>`.

For [custom domain](https://docs.codeberg.org/codeberg-pages/using-custom-domain/), create the `.domains` file inside the `./static/` directory so that it will be copied to the resulting build.

More information about Codeberg Pages is available in the [official Codeberg documentation](https://docs.codeberg.org/codeberg-pages/).
