+++
title = "Vercel"
weight = 50
+++

Vercel (previously Zeit) is similar to Netlify, making the deployment of your site easy as pie.
The sites are hosted by Vercel and automatically deployed whenever we push a commit to our
selected production branch (e.g, master).

If you don't have an account with Vercel, you can sign up [here](https://vercel.com/signup).

## Automatic deploys

Once you sign up you can import your site from a Git provider (Github, GitLab or Bitbucket). 
When you import your repository, Vercel will try to find out what framework your site is using.

If it doesn't default to Zola:
- Set Framework Preset as **Zola**.

By default, Vercel chooses output directory as `public`. If you use a different directory, then
specify output directory under the "Build and Output Settings" dropdown.
You can learn more about how to setup a custom domain and how to get the most out of Vercel
[via their documentation.](https://vercel.com/docs) 

After you click the blue "Deploy" button, it's off to the races!
