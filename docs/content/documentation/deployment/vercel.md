+++
title = "Vercel"
weight = 20
+++

Vercel (previously zeit) is same as Netlify which makes deployment of site easy.
The sites are hosted by Vercel and automatically deployed whenever we push a
commit to our selected production branch (e.g, master).

If you don't have an account with Vercel, you can sign up [here](https://vercel.com/signup)
## Automatic deploys

Once you sign up you can import your site from a Git provider (Github, GitLab or Bitbucket). 
After import you can select settings for your project. 

  - Choose Framework Preset as **Other**
  - Build command as `zola build` and make sure toggle on Override switch.
  - By default Vercel chooses output directory as `public`, if you have
    different out directory then specify output directory.
  - To add own domain, go to domain setting in left and add your domain.


All we have to is include a `vercel.json` in our projects root directory by
specifying `ZOLA_VERSION` we want to use to deploy the site.

```
  {
  "build": {
    "env": {
      "ZOLA_VERSION": "0.11.0"
    }
  }
}
```

And now your site is up and running.
