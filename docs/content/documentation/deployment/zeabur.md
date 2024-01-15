+++
title = "Zeabur"
weight = 50
+++

In this tutorial, we'll guide you through the process of deploying your Zola site onto the [Zeabur](https://zeabur.com). Zeabur provides a seamless deployment experience, with automatic SSL certificates and global-edge network delivery. Let's get started!

## Prerequisites

- A Zola site project on your local machine.
- A GitHub account and your Zola site project repository hosted on GitHub.
- A Zeabur account. If you don't have one, sign up at [here](https://zeabur.com/login).

## Step 1: Create a Project on Zeabur

1. Log in to your Zeabur account.
2. Navigate to the dashboard and click on the **Create Project** button.
3. Follow the on-screen instructions to set up a new project.

## Step 2: Push Your Zola Files to GitHub

1. Initialize a Git repository in your Zola project folder if you haven't already:

    ```bash
    git init
    git add .
    git commit -m "Initial commit"
    ```

2. Push your Zola project to GitHub:

    ```bash
    git remote add origin <your-github-repo-url>
    git branch -M main
    git push -u origin main
    ```

Replace `<your-github-repo-url>` with the URL of your GitHub repository.

## Step 3: Create a Service on Zeabur

1. Back in your Zeabur dashboard, click on **Create Service**.
2. Choose the **git** option to connect your GitHub repository.

## Step 4: Select Your Zola Repository

1. From the list of repositories, select the repository where your Zola project is located.

## Step 5: Automatic Deployment

Zeabur will automatically detect that you're deploying a Zola project and will handle the deployment process for you without any additional configuration needed.

To use a specific version of Zola, set [`ZOLA_VERSION`](https://zeabur.com/docs/environment/variables) environment variable in project settings to a valid
release tag, for example `0.17.2`.

## Step 6: Domain Binding

1. Once the deployment is complete, bind a domain name to your service.
2. You can choose to use a free `.zeabur.app` subdomain or bind your own custom domain.
3. Zeabur will automatically provide a free SSL certificate for your domain, ensuring secure browsing for your visitors.

## Step 7: Your Site is Live!

Congratulations! Your Zola site is now deployed and live, served through Zeabur's edge network. 

You can now visit your website at the domain you've set up.
