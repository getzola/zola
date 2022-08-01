+++
title = "Sourcehut Pages"
weight = 15
+++

Deploying your static Zola website on [Sourcehut Pages][srht] is very simple.

You need to create a `.build.yml` manifest file in the root folder of your Zola project and push your changes to the
Sourcehut git/hg repository.
To create your `.build.yml` file you can start with [a template][srht-tpl] or use the following example:
``` yaml
image: alpine/edge
packages:
  - hut
  - zola
oauth: pages.sr.ht/PAGES:RW
environment:
  site: your_username.srht.site
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
      hut pages publish -d $site site.tar.gz
```

This manifest will clone your source code, build the website and upload the generated static files to the domain
you specified in `site`.
For publishing the website, the build manifest uses `hut`, a commandline tool which takes care of automatically
generating authentication tokens, so there is nothing else you need to do.

From this template you need to customize the variable `site` with the domain that will host your website and
`sources` to point to your Sourcehut git/hg public URL (in this example `my-website` is the name of the repository).

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

The build job will be automatically triggered.
Notice that Sourcehut returns a direct link to the build page, where you can check the progress and success status.

By default you can use a subdomain of Sourcehut Pages to host your static website - `your_username.srht.site`.
If you want to use a custom domain (e.g. "blog.mydomain.org"), you will need to configure a DNS record to point to
the Sourcehut server.
Instructions on how to do this are available on [Sourcehut][srht-custom-domain].

[srht]: https://srht.site
[srht-tpl]: https://git.sr.ht/~sircmpwn/pages.sr.ht-examples
[srht-custom-domain]: https://srht.site/custom-domains
