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

To use a specific version of Zola, set [`ZOLA_VERSION`](https://vercel.com/docs/deployments/environments#specifying-framework-versions) environment variable in project settings to a valid
release tag, for example `0.17.2`.

## Troubleshooting

### `GLIBC_X.XX` not found

This is because Vercel's build images comes with an older glibc version whereas Zola 
depends on a newer glibc. However, Vercel provides a newer build image which can be used in
deployments by setting Node.js version to "20.x", allowing Zola to work properly.

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
if you have an `about.md` file, Zola will generate a `about/index.html` file, but Vercel
will serve the file as `/about`, without its `index.html` suffix.

To enable that, create a file in the root of your git repository named `vercel.json`
(if it doesn't exists already), and set this option:

```json
{
    "cleanUrls": true
}
```

### Using a custom Zola binary

If you want to use your own Zola binary that published on GitHub, or if you want to
always use the latest version of Zola, you can run a shell command to grab the
latest release from GitHub.

To do that, set "Framework Preset" to "Other", and override "Install Command" to:

```bash
REPO="getzola/zola"; curl -fsS https://api.github.com/repos/${REPO}/releases/latest | grep -oP '"browser_download_url": ?"\K(.+linux-gnu.tar.gz)' | xargs -n 1 curl -fsSL -o zola.tar.gz && tar -xzvf zola.tar.gz
```

This command will fetch the latest release from GitHub, download the archive and extract it.

Then, set "Build Command" to `./zola build`. Now Vercel will use the downloaded Zola 
binary to build the documentation instead of using the built-in one.

If you prefer to use `vercel.json` instead, (which overrides the options set in the dashboard)
you can use this configuration.

```json
{
    "framework": null,
    "installCommand": "REPO=\"getzola/zola\"; curl -fsS https://api.github.com/repos/${REPO}/releases/latest | grep -oP '\"browser_download_url\": ?\"\\K(.+linux-gnu.tar.gz)' | xargs -n 1 curl -fsSL -o zola.tar.gz && tar -xzvf zola.tar.gz",
    "buildCommand": "./zola build",
    "outputDirectory": "public"
}
```

## See also

See [Vercel's own documentation](https://vercel.com/docs/projects/project-configuration) 
for all available options in `vercel.json`.