+++
title = "Sourcehut Pages"
weight = 15
+++

Deploying your static Zola website on [Sourcehut Pages][srht] is very simple.

You only need to create a manifest `.build.yml` file in your root folder of your Zola project and push your changes to the Sourcehut git/hg repository. To create your `.build.yml` you can start by [a template][srht-tpl].

Example:

``` yaml
image: alpine/edge
packages: [ zola ]
oauth: pages.sr.ht/PAGES:RW
environment:
  site: www.example.org
sources:
  - https://git.sr.ht/~your_username/my-website
tasks:
  - build: |
      cd my-website
      zola build
  - package: |
      cd my-website
      tar -C public -cvz . > ../site.tar.gz
  - upload: |
      acurl -f https://pages.sr.ht/publish/$site -Fcontent=@site.tar.gz
```

This manifest will checkout your code from `sources`, build and upload the generated static files to `site` using a wrapper script around `curl` (called `acurl`, already available in all Sourcehut builds).

From this template you need to customize the variable `site` with the domain that will host your website and `sources` to point to your Sourcehut git/hg public URL (in this example `my-website` is the name of the repository).

Then commit and push your changes:

``` sh
$ git push
Enumerating objects: 5, done.
...
remote: Build started:
remote: https://builds.sr.ht/~your_username/job/430625 [.build.yml]
To git.sr.ht:~your_username/www
   fbe9afa..59ae556  master -> master
```

The build job will be automatically triggered. Notice that Sourcehut returns a direct link to the build page.

By default you can use a subdomain of Sourcehut Pages to host your static website (e.g. "your_username.srht.site"). If you want to use a custom domain (e.g. "blog.mydomain.org"), you will need to configure a DNS record to point to the Sourcehut server. Instructions to do this are detailed on [Sourcehut][srht-custom-domain].

[srht]: https://srht.site
[srht-tpl]: https://git.sr.ht/~sircmpwn/pages.sr.ht-examples
[srht-custom-domain]: https://srht.site/custom-domains
