+++
title = "GitLab Pages"
weight = 40
+++

We are going to use the GitLab Runner in GitLab CI/CD to host
the site on GitLab Pages.

## Configuring your repository

It is possible to host your Zola site on either the SaaS instance offered by
GitLab (<https://gitlab.com>) or on a self-hosted instance.

It is recommended to create a repository on GitLab that contains solely your
Zola project. The [Zola's directory structure](https://www.getzola.org/documentation/getting-started/directory-structure/)
should be located in the root directory of your repository.

For information on how to create and manage a repository on GitLab, refer to:

<https://docs.gitlab.com/ee/user/project/repository/>

## Ensuring that the runner can access your theme

Depending on how you added your theme, your repository may not contain it.
The best way to ensure that the theme will be added is to use submodules.
When doing this, ensure that you are using the `https` version of the URL.

```bash
git submodule add {THEME_URL} themes/{THEME_NAME}
```

For example, this could look like:

```bash
git submodule add https://github.com/getzola/hyde.git themes/hyde
```

## Setting up the GitLab Runner

The GitLab Runner needs to know how to create your site in order to deploy
it to the GitLab Pages server.

We provide you with a template to accomplish this task easily.
Create a file called `.gitlab-ci.yml` in the root directory of your
repository and copy the contents of the template below.

```yaml
stages:
  - deploy

default:
  image: debian:stable-slim
  tags:
    - docker

variables:
  # The runner will be able to pull your Zola theme when the strategy is
  # set to "recursive".
  GIT_SUBMODULE_STRATEGY: "recursive"

  # If you don't set a version here, your site will be built with the latest
  # version of Zola available in GitHub releases.
  # Use the semver (x.y.z) format to specify a version. For example: "0.17.2" or "0.18.0".
  ZOLA_VERSION:
    description: "The version of Zola used to build the site."
    value: ""

pages:
  stage: deploy
  script:
    - |
      apt-get update --assume-yes && apt-get install --assume-yes --no-install-recommends wget ca-certificates
      if [ $ZOLA_VERSION ]; then
        zola_url="https://github.com/getzola/zola/releases/download/v$ZOLA_VERSION/zola-v$ZOLA_VERSION-x86_64-unknown-linux-gnu.tar.gz"
        if ! wget --quiet --spider $zola_url; then
          echo "A Zola release with the specified version could not be found.";
          exit 1;
        fi
      else
        github_api_url="https://api.github.com/repos/getzola/zola/releases/latest"
        zola_url=$(
          wget --output-document - $github_api_url |
          grep "browser_download_url.*linux-gnu.tar.gz" |
          cut --delimiter : --fields 2,3 |
          tr --delete "\" "
        )
      fi
      wget $zola_url
      tar -xzf *.tar.gz
      ./zola build

  artifacts:
    paths:
      # This is the directory whose contents will be deployed to the GitLab Pages
      # server.
      # GitLab Pages expects a directory with this name by default.
      - public

  rules:
    # This rule makes it so that your website is published and updated only when
    # you push to the default branch of your repository (e.g. "master" or "main").
    - if: $CI_COMMIT_BRANCH == $CI_DEFAULT_BRANCH
```

Please, keep in mind that this template assumes you are using the
[Docker executor](https://docs.gitlab.com/runner/executors/docker.html)
on your GitLab Runner.
Feel free to adjust the file to your workflow and specific requirements.

After you push this file to the default branch of your repository
(e.g. "master" or "main"), your site will be ready. The GitLab CI/CD pipelines
will ensure your site is published and updated automatically.

On the left sidebar of GitLab, navigate to **Deploy > Pages** to find the URL of your
website inside the **Access pages** section.

More information on how to host your site on GitLab Pages is available
[in the official GitLab documentation](https://docs.gitlab.com/ee/user/project/pages/).
