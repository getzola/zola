+++
title = "Azure Static Web Apps"
weight = 60
+++

[Azure Static Web Apps](https://learn.microsoft.com/en-us/azure/static-web-apps/overview) is a managed Azure service that handles automatically deploying static site content from changes to GitHub or Azure DevOps repositories. We'll be going over how to configure deploying your site to Azure Static Web Apps via GitHub Actions.

### Setting up Azure Static Web Apps
Follow the [official documentation](https://learn.microsoft.com/en-us/azure/static-web-apps/get-started-portal?tabs=vanilla-javascript&pivots=github) for configuring the static web app in the Azure portal with the `GitHub` as the selected code hosting platform except for the `Build Details section.

Instead, for the`Build Details` section, set the App location as `./public` since that is where `zola build` will write the site content to by default. Leave the other boxes empty.

After creating the web app, make note of the domain automatically created by Azure and update `base_url` in your repo's `config.toml` to that URL.


### Configuring GitHub Actions
Azure should have already created a GitHub Actions YAML file in your GitHub repository under `.github/workflows` and configured secrets for deploying your app to Azure. Now we'll need to update the workflow to install Zola and build your site. 


```yaml
# .github/workflows/azure-static-web-apps-<web-app-name>
name: Azure Static Web Apps CI/CD

on:
  push:
    branches:
      - main
  pull_request:
    types: [opened, synchronize, reopened, closed]
    branches:
      - main

jobs:
  build_and_deploy_job:
    if: github.event_name == 'push' || (github.event_name == 'pull_request' && github.event.action != 'closed')
    runs-on: ubuntu-latest
    name: Build and Deploy Job
    steps:
      - uses: actions/checkout@v3
        with:
          submodules: true
          lfs: false
      - uses: taiki-e/install-action@v2
        with:
          tool: zola@0.21.0
      - name: Build Static Site
        run: zola build
      - name: Build And Deploy
        id: builddeploy
        uses: Azure/static-web-apps-deploy@v1
        with:
          azure_static_web_apps_api_token: ${{ secrets.AZURE_STATIC_WEB_APPS_API_TOKEN_<WEB_APP_NAME>}}
          repo_token: ${{ secrets.GITHUB_TOKEN }} # Used for Github integrations (i.e. PR comments)
          action: "upload"
          ###### Repository/Build Configurations - These values can be configured to match your app requirements. ######
          # For more information regarding Static Web App workflow configurations, please visit: https://aka.ms/swaworkflowconfig
          app_location: "./public" # App source code path
          api_location: "" # Api source code path - optional
          output_location: "" # Built app content directory - optional
          ###### End of Repository/Build Configurations ######

  close_pull_request_job:
    if: github.event_name == 'pull_request' && github.event.action == 'closed'
    runs-on: ubuntu-latest
    name: Close Pull Request Job
    steps:
      - name: Close Pull Request
        id: closepullrequest
        uses: Azure/static-web-apps-deploy@v1
        with:
          azure_static_web_apps_api_token: ${{ secrets.AZURE_STATIC_WEB_APPS_API_TOKEN<WEB_APP_NAME> }}
          action: "close"

```
Once your YAML changes have been pushed, GitHub will automatically kick off a workflow deploying your site!