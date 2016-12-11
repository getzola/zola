# Gutenberg

## Design

Can be used for blogs or general static pages

Commands:

- new: start a new project -> creates the structure + default config.toml
- build: reads all the .md files and build the site with template
- serve: starts a server and watches/reload the site on change


All pages go into the `content` folder. Subfolder represents a list of content, ie

```bash
├── content
│   ├── posts
│   │   └── intro.md
│   └── some.md
```

`some.md` will be accessible at `mywebsite.com/some` and there will be other pages:

- `mywebsite.com/posts` that will list all the pages contained in the `posts` folder
- `mywebsite.com/posts/intro`


### Building the site
Get all .md files in content, remove the `content/` prefix to their path
Split the file between front matter and content
Parse the front matter
markdown -> HTML for the content
TO THINK OF: create list pages for folders, can be done while globbing I guess?
Render templates
