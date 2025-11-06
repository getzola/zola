+++
title = "Vercel"
weight = 50
+++

Vercel (previously Zeit) is similar to Netlify, making the deployment of your site easy as pie.
The sites are hosted by Vercel and automatically deployed whenever we push a commit to our
selected production branch (e.g, master).

If you don't have an account with Vercel, you can sign up [here.](https://vercel.com/signup)

## Automatic deploys

Once you sign up you can import your site from a Git provider (Github, GitLab or Bitbucket). 
When you import your repository, Vercel will try to find out what framework your site is using.

If it doesn't default to Zola:

- Set "Framework Preset" as **Zola**.

By default, Vercel chooses output directory as `public`. If you use a different directory, then
specify output directory under the "Build and Output Settings" dropdown.
You can learn more about how to setup a custom domain and how to get the most out of Vercel
[via their documentation.](https://vercel.com/docs) 

After you click the blue "Deploy" button, it's off to the races!

To use a specific version of Zola, set [`ZOLA_VERSION`](https://vercel.com/docs/deployments/environments#specifying-framework-versions) environment variable in project settings to a valid
release tag, for example `0.17.2`.

## Troubleshooting

### `GLIBC_X.XX` not found

This is because Vercel's build images doesn't come with a `glibc` version that Zola requires. 
Vercel provides [different build images](https://vercel.com/docs/builds/build-image) for
different Node.js versions, so even though Zola has no relation with Node.js at all, you can
bump your Node.js version from project settings to make Vercel to use a newer build
environment, allowing Zola to work properly.

If your project was created before when the default Node.js version was `20.x`, bumping
Node.js version to `22.x` (which is the new default) should work for Zola versions up to
`0.19.2` (inclusive) without further configuration.

Since Vercel does not provide a even newer image, subsequent versions of Zola will not work
again when using the built-in Zola framework preset. However, starting with Zola version
`0.21.0`, a statically linked `musl` binary being released, which provides the highest
compatibility among systems where glibc is insufficient - like Vercel's build images. To use
the `musl`-compiled binary, you must ensure that Vercel
[does not use its built-in Zola preset and instead provide the binary yourself.](#using-a-custom-zola-binary)

## Additional options

### Enable trailing slashes

Visiting a page without trailing slash may break relative paths, so you might want to configure
Vercel to always redirect paths with a trailing slash. By default, redirecting to a trailing
slash is not enabled on Vercel.

For example if you have an `about.md` file, and when visiting the path without a trailing
slash, like `/about`, Vercel will redirect with trailing slash, resulting in `/about/`.
Paths with a file extension will not redirect to a trailing slash, for example if you
have a static file named `favicon.ico`, it will stay as-is.

To enable that, create a file in the root of your git repository named `vercel.json`
(if it doesn't exists already), and set this option:

```json
{
    "trailingSlash": true
}
```

### Prefer clean URLs

When enabled, all HTML files will be served without their file extension. For example
if you have an `about.md` file, Zola will generate a `about/index.html` file, but you may
prefer to have Vercel to serve the file as `/about` without its `index.html` suffix.

To enable that, create a file in the root of your git repository named `vercel.json`
(if it doesn't exists already), and set this option:

```json
{
    "cleanUrls": true
}
```

### Using a custom Zola binary

If you want to provide Zola binary on your own for full control instead of having Vercel
to use their controlled Zola preset, set the "Framework Preset" to "Other". This will make
Vercel to not get use of `ZOLA_VERSION` variable anymore automatically.
Then, set "Install Command" to:

```bash
echo "${ZOLA_VERSION:-"latest"}" | sed '/^latest$/!s/\(.*\)/tags\/v\1/' | xargs -I% curl -fsSL "https://api.github.com/repos/getzola/zola/releases/%" | grep -oP "\"browser_download_url\": ?\"\\K(.+linux-${ZOLA_LIBC:-"musl"}\\.tar\\.gz)" | xargs curl -fsSL | tar -xz
```

This command will download Zola from the file URL obtained from GitHub API and extract it.
This way we can continue using same `ZOLA_VERSION` environment variable name to pin to a
specific Zola version (same as how Vercel does) - or set it to `latest` to always pull the
latest version whenever an deployment is initiated on Vercel.

We also pull `musl` binaries by default in the command, so we no longer have to worry
about Vercel's build images. But if you would like to use older versions (below of
`0.21.0` where there isn't a `musl` binary provided), you need to create a new environment
variable named `ZOLA_LIBC` and set it to `gnu`.

Along with setting "Install Command" to above, you will also need to set "Build Command"
to `./zola build`, so we can have our site built with the locally downloaded Zola binary
previously.

If you prefer to use `vercel.json` instead,
(which overrides the options set in the dashboard) you can use this configuration:

```json
{
    "framework": null,
    "installCommand": "echo \"${ZOLA_VERSION:-\"latest\"}\" | sed '/^latest$/!s/\\(.*\\)/tags\\/v\\1/' | xargs -I% curl -fsSL \"https://api.github.com/repos/getzola/zola/releases/%\" | grep -oP \"\\\"browser_download_url\\\": ?\\\"\\K(.+linux-${ZOLA_LIBC:-\"musl\"}\\\\.tar\\\\.gz)\" | xargs curl -fsSL | tar -xz",
    "buildCommand": "./zola build",
    "outputDirectory": "public"
}
```

You can modify the commands as your wish if you would like to use your own fork and use
binaries released there.

## See also

See [Vercel's own documentation](https://vercel.com/docs/projects/project-configuration) 
for all available options in `vercel.json`.