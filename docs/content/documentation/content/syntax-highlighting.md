+++
title = "Syntax Highlighting"
weight = 80
+++

Zola comes with built-in syntax highlighting but you first
need to enable it in the [configuration](@/documentation/getting-started/configuration.md).

Once this is done, Zola will automatically highlight all code blocks
in your content. A code block in Markdown looks like the following:

````
```rust
let highlight = true;
```
````

You can replace `rust` with another language or not put anything to get the text
interpreted as plain text.

Here is a full list of supported languages and their short names:

```
- ActionScript -> ["as"]
- Advanced CSV -> ["csv", "tsv"]
- AppleScript -> ["applescript", "script editor"]
- ASP -> ["asa"]
- Assembly x86 (NASM) -> ["asm", "inc", "nasm"]
- AWK -> ["awk"]
- Batch File -> ["bat", "cmd"]
- BibTeX -> ["bib"]
- Bourne Again Shell (bash) -> [".bash_aliases", ".bash_completions", ".bash_functions", ".bash_login", ".bash_logout", ".bash_profile", ".bash_variables", ".bashrc", ".ebuild", ".eclass", ".profile", ".textmate_init", ".zlogin", ".zlogout", ".zprofile", ".zshenv", ".zshrc", "PKGBUILD", "ash", "bash", "sh", "zsh"]
- C -> ["c", "h"]
- C# -> ["cs", "csx"]
- C++ -> ["C", "c++", "cc", "cp", "cpp", "cxx", "h", "h++", "hh", "hpp", "hxx", "inl", "ipp"]
- Clojure -> ["clj", "cljc", "cljs", "edn"]
- ClojureC -> ["boot", "clj", "cljc", "cljs", "cljx"]
- CMake -> ["CMakeLists.txt", "cmake"]
- CMake C Header -> ["h.in"]
- CMake C++ Header -> ["h++.in", "hh.in", "hpp.in", "hxx.in"]
- CMakeCache -> ["CMakeCache.txt"]
- Crystal -> ["cr"]
- CSS -> ["css", "css.erb", "css.liquid"]
- D -> ["d", "di"]
- Dart -> ["dart"]
- Diff -> ["diff", "patch"]
- Dockerfile -> ["Dockerfile", "dockerfile"]
- EDN -> ["edn"]
- Elixir -> ["ex", "exs"]
- Elm -> ["elm"]
- Erlang -> ["Emakefile", "emakefile", "erl", "escript", "hrl"]
- F# -> ["fs", "fsi", "fsx"]
- Fortran (Fixed Form) -> ["F", "F77", "FOR", "FPP", "f", "f77", "for", "fpp"]
- Fortran (Modern) -> ["F03", "F08", "F90", "F95", "f03", "f08", "f90", "f95"]
- Fortran Namelist -> ["namelist"]
- Friendly Interactive Shell (fish) -> ["fish"]
- GDScript (Godot Engine) -> ["gd"]
- Generic Config -> [".dircolors", ".gitattributes", ".gitignore", ".gitmodules", ".inputrc", "Doxyfile", "cfg", "conf", "config", "dircolors", "gitattributes", "gitignore", "gitmodules", "ini", "inputrc", "mak", "mk", "pro"]
- Git Attributes -> [".gitattributes", "attributes", "gitattributes"]
- Git Commit -> ["COMMIT_EDITMSG", "MERGE_MSG", "TAG_EDITMSG"]
- Git Config -> [".gitconfig", ".gitmodules", "gitconfig"]
- Git Ignore -> [".gitignore", "exclude", "gitignore"]
- Git Link -> [".git"]
- Git Log -> ["gitlog"]
- Git Mailmap -> [".mailmap", "mailmap"]
- Git Rebase Todo -> ["git-rebase-todo"]
- GLSL -> ["comp", "frag", "fs", "fsh", "fshader", "geom", "glsl", "gs", "gsh", "gshader", "tesc", "tese", "vert", "vs", "vsh", "vshader"]
- Go -> ["go"]
- GraphQL -> ["gql", "graphql", "graphqls"]
- Graphviz (DOT) -> ["DOT", "dot", "gv"]
- Groovy -> ["Jenkinsfile", "gradle", "groovy", "gvy"]
- Handlebars -> ["handlebars", "handlebars.html", "hbr", "hbrs", "hbs", "hdbs", "hjs", "mu", "mustache", "rac", "stache", "template", "tmpl"]
- Haskell -> ["hs"]
- HTML -> ["htm", "html", "shtml", "xhtml"]
- HTML (ASP) -> ["asp"]
- HTML (EEx) -> ["html.eex", "html.leex"]
- HTML (Erlang) -> ["yaws"]
- HTML (Jinja2) -> ["htm.j2", "html.j2", "xhtml.j2", "xml.j2"]
- HTML (Rails) -> ["erb", "html.erb", "rails", "rhtml"]
- HTML (Tcl) -> ["adp"]
- Java -> ["bsh", "java"]
- Java Properties -> ["properties"]
- Java Server Page (JSP) -> ["jsp"]
- JavaScript -> ["htc", "js"]
- JavaScript (Rails) -> ["js.erb"]
- Jinja2 -> ["j2", "jinja", "jinja2"]
- JSON -> ["Pipfile.lock", "ipynb", "json", "sublime-build", "sublime-color-scheme", "sublime-commands", "sublime-completions", "sublime-keymap", "sublime-macro", "sublime-menu", "sublime-mousemap", "sublime-project", "sublime-settings", "sublime-theme"]
- Julia -> ["jl"]
- Kotlin -> ["kt", "kts"]
- LaTeX -> ["ltx", "tex"]
- Less -> ["css.less", "less"]
- Linker Script -> ["ld"]
- Lisp -> ["cl", "clisp", "el", "fasl", "l", "lisp", "lsp", "mud", "scm", "ss"]
- Literate Haskell -> ["lhs"]
- lrc -> ["lrc", "lyric"]
- Lua -> ["lua"]
- Makefile -> ["GNUmakefile", "Makefile", "Makefile.am", "Makefile.in", "OCamlMakefile", "mak", "make", "makefile", "makefile.am", "makefile.in", "mk"]
- Markdown -> ["markdn", "markdown", "md", "mdown"]
- MATLAB -> ["matlab"]
- MiniZinc (MZN) -> ["dzn", "mzn"]
- NAnt Build File -> ["build"]
- Nim -> ["nim", "nims"]
- Nix -> ["nix"]
- Objective-C -> ["h", "m"]
- Objective-C++ -> ["M", "h", "mm"]
- OCaml -> ["ml", "mli"]
- OCamllex -> ["mll"]
- OCamlyacc -> ["mly"]
- Pascal -> ["dpr", "p", "pas"]
- Perl -> ["pc", "pl", "pm", "pmc", "pod", "t"]
- PHP -> ["php", "php3", "php4", "php5", "php7", "phps", "phpt", "phtml"]
- Plain Text -> ["txt"]
- PowerShell -> ["ps1", "psd1", "psm1"]
- Protocol Buffer -> ["proto", "protodevel"]
- Protocol Buffer (TEXT) -> ["pb.txt", "pbtxt", "proto.text", "prototxt", "textpb"]
- PureScript -> ["purs"]
- Python -> ["SConscript", "SConstruct", "Sconstruct", "Snakefile", "bazel", "bzl", "cpy", "gyp", "gypi", "pxd", "pxd.in", "pxi", "pxi.in", "py", "py3", "pyi", "pyw", "pyx", "pyx.in", "rpy", "sconstruct", "vpy", "wscript"]
- R -> ["R", "Rprofile", "r"]
- Racket -> ["rkt"]
- Rd (R Documentation) -> ["rd"]
- Reason -> ["re", "rei"]
- Regular Expression -> ["re"]
- Regular Expressions (Elixir) -> ["ex.re"]
- reStructuredText -> ["rest", "rst"]
- Ruby -> ["Appfile", "Appraisals", "Berksfile", "Brewfile", "Cheffile", "Deliverfile", "Fastfile", "Gemfile", "Guardfile", "Podfile", "Rakefile", "Rantfile", "Scanfile", "Snapfile", "Thorfile", "Vagrantfile", "capfile", "cgi", "config.ru", "fcgi", "gemspec", "irbrc", "jbuilder", "podspec", "prawn", "rabl", "rake", "rb", "rbx", "rjs", "ruby.rail", "simplecov", "thor"]
- Ruby Haml -> ["haml", "sass"]
- Ruby on Rails -> ["builder", "rxml"]
- Rust -> ["rs"]
- Sass -> ["sass"]
- Scala -> ["sbt", "sc", "scala"]
- SCSS -> ["scss"]
- SQL -> ["ddl", "dml", "sql"]
- SQL (Rails) -> ["erbsql", "sql.erb"]
- srt -> ["srt", "subrip"]
- Stylus -> ["styl", "stylus"]
- SWI-Prolog -> ["pro"]
- Swift -> ["swift"]
- Tcl -> ["tcl"]
- TeX -> ["cls", "sty"]
- Textile -> ["textile"]
- TOML -> ["Cargo.lock", "Gopkg.lock", "Pipfile", "tml", "toml"]
- TypeScript -> ["ts"]
- TypeScriptReact -> ["tsx"]
- VimL -> ["vim"]
- XML -> ["dtml", "opml", "rng", "rss", "svg", "tld", "xml", "xsd", "xslt"]
- YAML -> ["sublime-syntax", "yaml", "yml"]
- Zig -> ["zig"]
```

Note: due to some issues with the JavaScript syntax, the TypeScript syntax will be used instead.

If the language you want to highlight is not on this list, the `extra_syntaxes_and_themes` configuration option can be used to add additional syntax and theme files.

If your site source is laid out as follows:

```
.
├── config.toml
├── content/
│   └── ...
├── static/
│   └── ...
├── syntaxes/
│   ├── Sublime-Language1/
│   │   └── lang1.sublime-syntax
│   └── lang2.sublime-syntax
└── templates/
    └── ...
```

you would set your `extra_syntaxes_and_themes` to `["syntaxes", "syntaxes/Sublime-Language1"]` to load `lang1.sublime-syntax` and `lang2.sublime-syntax`.

You can see the list of available themes on the [configuration page](@/documentation/getting-started/configuration.md#syntax-highlighting).


## Inline VS classed highlighting

If you use a highlighting scheme like

```toml
highlight_theme = "base16-ocean-dark"
```

for a code block like

````md
```rs
let highlight = true;
```
````

you get the colors directly encoded in the html file.

```html
<pre class="language-rs" style="background-color:#2b303b;">
    <code class="language-rs">
        <span style="color:#b48ead;">let</span>
        <span style="color:#c0c5ce;"> highlight = </span>
        <span style="color:#d08770;">true</span>
        <span style="color:#c0c5ce;">;
    </span>
  </code>
</pre>
```

This is nice, because your page will load faster if everything is in one file.
But if you would like to have the user choose a theme from a
list, or use different color schemes for dark/light color schemes, you need a
different solution.

If you use the special `css` color scheme

```toml
highlight_theme = "css"
```

you get CSS class definitions, instead.

```html
<pre class="language-rs">
    <code class="language-rs">
        <span class="z-source z-rust">
            <span class="z-storage z-type z-rust">let</span> highlight
            <span class="z-keyword z-operator z-assignment z-rust">=</span>
            <span class="z-constant z-language z-rust">true</span>
            <span class="z-punctuation z-terminator z-rust">;</span>
        </span>
    </code>
</pre>
```

Zola can output a css file for a theme in the `static` directory using the `highlight_themes_css` option.

```toml
highlight_themes_css = [
  { theme = "base16-ocean-dark", filename = "syntax-theme-dark.css" },
  { theme = "base16-ocean-light", filename = "syntax-theme-light.css" },
]
```

You can then support light and dark mode like so:

```css
@import url("syntax-theme-dark.css") (prefers-color-scheme: dark);
@import url("syntax-theme-light.css") (prefers-color-scheme: light);
```

Alternately, you can reference the stylesheets in your base template to reduce request chains:

```html
<head>
  <!-- Other content -->
  <link rel="stylesheet" type="text/css" href="/syntax-theme-dark.css" media="(prefers-color-scheme: dark)" />
  <link rel="stylesheet" type="text/css" href="/syntax-theme-light.css" media="(prefers-color-scheme: light)" />
</head>
```

Themes can conditionally include code-highlighting stylesheet `<link>` tags by wrapping them in a conditional:

```jinja2
{% if config.markdown.highlight_code and config.markdown.highlight_theme == "css" %}
<link rel="stylesheet" type="text/css" href="/syntax-theme-dark.css" media="(prefers-color-scheme: dark)" />
<link rel="stylesheet" type="text/css" href="/syntax-theme-light.css" media="(prefers-color-scheme: light)" />
{% endif %}
```


## Annotations

You can use additional annotations to customize how code blocks are displayed:

- `linenos` to enable line numbering.

````
```rust,linenos
use highlighter::highlight;
let code = "...";
highlight(code);
```
````

- `linenostart` to specify the number for the first line (defaults to 1)
  
````
```rust,linenos,linenostart=20
use highlighter::highlight;
let code = "...";
highlight(code);
```
````

- `hl_lines` to highlight lines. You must specify a list of inclusive ranges of lines to highlight,
separated by ` ` (whitespace). Ranges are 1-indexed and `linenostart` doesn't influence the values, it always refers to the codeblock line number.
  
````
```rust,hl_lines=1 3-5 9
use highlighter::highlight;
let code = "...";
highlight(code);
```
````

- `hide_lines` to hide lines. You must specify a list of inclusive ranges of lines to hide,
separated by ` ` (whitespace). Ranges are 1-indexed.
  
````
```rust,hide_lines=1-2
use highlighter::highlight;
let code = "...";
highlight(code);
```
````

## Styling codeblocks

Depending on the annotations used, some codeblocks will be hard to read without any CSS. We recommend using the following
snippet in your sites:

```scss
pre {
  padding: 1rem;
  overflow: auto;
}
// The line numbers already provide some kind of left/right padding
pre[data-linenos] {
  padding: 1rem 0;
}
pre table td {
  padding: 0;
}
// The line number cells
pre table td:nth-of-type(1) {
  text-align: center;
  user-select: none;
}
pre mark {
  // If you want your highlights to take the full width.
  display: block;
  // The default background colour of a mark is bright yellow
  background-color: rgba(254, 252, 232, 0.9);
}
pre table {
  width: 100%;
  border-collapse: collapse;
}
```

This snippet makes the highlighting work on the full width and ensures that a user can copy the content without
selecting the line numbers. Obviously you will probably need to adjust it to fit your site style.

Here's an example with all the options used: `scss, linenos, linenostart=10, hl_lines=3-4 8-9, hide_lines=2 7` with the
snippet above.

```scss, linenos, linenostart=10, hl_lines=3-4 8-9, hide_lines=2 7
pre mark {
  // If you want your highlights to take the full width.
  display: block;
  color: currentcolor;
}
pre table td:nth-of-type(1) {
  // Select a colour matching your theme
  color: #6b6b6b;
  font-style: italic;
}
```

Line 2 and 7 are comments that are not shown in the final output.

When line numbers are active, the code block is turned into a table with one row and two cells. The first cell contains the line number and the second cell contains the code.
Highlights are done via the `<mark>` HTML tag. When a line with line number is highlighted two `<mark>` tags are created: one around the line number(s) and one around the code.

## Custom Highlighting Themes

The default *theme* for syntax highlighting is called `base16-ocean-dark`, you can choose another theme from the built in set of highlight themes using the `highlight_theme` configuration option.
For example, this documentation site currently uses the `kronuz` theme, which is built in.

```
[markdown]
highlight_code = true
highlight_theme = "kronuz"
```

Alternatively, the `extra_syntaxes_and_themes` configuration option can be used to add additional theme files.
You can load your own highlight theme from a TextMate `.tmTheme` file.

It works the same way as adding extra syntaxes. It should contain a list of paths to folders containing the .tmTheme files you want to include.
You would then set `highlight_theme` to the name of one of these files, without the `.tmTheme` extension.

If your site source is laid out as follows:

```
.
├── config.toml
├── content/
│   └── ...
├── static/
│   └── ...
├── highlight_themes/
│   ├── MyGroovyTheme/
│   │   └── theme1.tmTheme
│   ├── theme2.tmTheme
└── templates/
    └── ...
```

you would set your `extra_syntaxes_and_themes` to `["highlight_themes", "highlight_themes/MyGroovyTheme"]` to load `theme1.tmTheme` and `theme2.tmTheme`.
Then choose one of them to use, say theme1, by setting `highlight_theme = theme1`.
