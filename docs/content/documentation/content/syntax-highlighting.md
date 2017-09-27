+++
title = "Syntax Highlighting"
weight = 80
+++

Gutenberg comes with built-in syntax highlighting but you first 
need to enable it in the [configuration](./documentation/getting-started/configuration.md).

Once this is done, Gutenberg will automatically highlight all code blocks
in your content. A code block in Markdown looks like the following:

````md

```rust
let highlight = true;
```

````

You can replace the `rust` by the language you want to highlight.
Here is a full list of the supported languages and the short name you can use:

```
- Plain Text -> ["txt"]
- Assembly x86 (NASM) -> ["asm", "inc", "nasm"]
- Elm -> ["elm"]
- Handlebars -> ["handlebars", "handlebars.html", "hbr", "hbrs", "hbs", "hdbs", "hjs", "mu", "mustache", "rac", "stache", "template", "tmpl"]
- Jinja2 -> ["j2", "jinja2"]
- Julia -> ["jl"]
- LESS -> ["less"]
- ASP -> ["asa"]
- HTML (ASP) -> ["asp"]
- ActionScript -> ["as"]
- AppleScript -> ["applescript", "script editor"]
- Batch File -> ["bat", "cmd"]
- NAnt Build File -> ["build"]
- C# -> ["cs", "csx"]
- C++ -> ["cpp", "cc", "cp", "cxx", "c++", "C", "h", "hh", "hpp", "hxx", "h++", "inl", "ipp"]
- C -> ["c", "h"]
- CSS -> ["css", "css.erb", "css.liquid"]
- Clojure -> ["clj"]
- D -> ["d", "di"]
- Diff -> ["diff", "patch"]
- Erlang -> ["erl", "hrl", "Emakefile", "emakefile"]
- HTML (Erlang) -> ["yaws"]
- Go -> ["go"]
- Graphviz (DOT) -> ["dot", "DOT"]
- Groovy -> ["groovy", "gvy", "gradle"]
- HTML -> ["html", "htm", "shtml", "xhtml", "inc", "tmpl", "tpl"]
- Haskell -> ["hs"]
- Literate Haskell -> ["lhs"]
- Java Server Page (JSP) -> ["jsp"]
- Java -> ["java", "bsh"]
- JavaDoc -> []
- Java Properties -> ["properties"]
- JSON -> ["json", "sublime-settings", "sublime-menu", "sublime-keymap", "sublime-mousemap", "sublime-theme", "sublime-build", "sublime-project", "sublime-completions", "sublime-commands", "sublime-macro"]
- JavaScript -> ["js", "htc"]
- Regular Expressions (Javascript) -> []
- BibTeX -> ["bib"]
- LaTeX Log -> []
- LaTeX -> ["tex", "ltx"]
- TeX -> ["sty", "cls"]
- Lisp -> ["lisp", "cl", "l", "mud", "el", "scm", "ss", "lsp", "fasl"]
- Lua -> ["lua"]
- Make Output -> []
- Makefile -> ["make", "GNUmakefile", "makefile", "Makefile", "OCamlMakefile", "mak", "mk"]
- Markdown -> ["md", "mdown", "markdown", "markdn"]
- MultiMarkdown -> []
- MATLAB -> ["matlab"]
- OCaml -> ["ml", "mli"]
- OCamllex -> ["mll"]
- OCamlyacc -> ["mly"]
- camlp4 -> []
- Objective-C++ -> ["mm", "M", "h"]
- Objective-C -> ["m", "h"]
- PHP Source -> []
- PHP -> ["php", "php3", "php4", "php5", "php7", "phps", "phpt", "phtml"]
- Pascal -> ["pas", "p", "dpr"]
- Perl -> ["pl", "pm", "pod", "t", "PL"]
- Python -> ["py", "py3", "pyw", "pyi", "rpy", "cpy", "SConstruct", "Sconstruct", "sconstruct", "SConscript", "gyp", "gypi", "Snakefile", "wscript"]
- Regular Expressions (Python) -> []
- R Console -> []
- R -> ["R", "r", "s", "S", "Rprofile"]
- Rd (R Documentation) -> ["rd"]
- HTML (Rails) -> ["rails", "rhtml", "erb", "html.erb"]
- JavaScript (Rails) -> ["js.erb"]
- Ruby Haml -> ["haml", "sass"]
- Ruby on Rails -> ["rxml", "builder"]
- SQL (Rails) -> ["erbsql", "sql.erb"]
- Regular Expression -> ["re"]
- reStructuredText -> ["rst", "rest"]
- Ruby -> ["rb", "Appfile", "Appraisals", "Berksfile", "Brewfile", "capfile", "cgi", "Cheffile", "config.ru", "Deliverfile", "Fastfile", "fcgi", "Gemfile", "gemspec", "Guardfile", "irbrc", "jbuilder", "podspec", "prawn", "rabl", "rake", "Rakefile", "Rantfile", "rbx", "rjs", "ruby.rail", "Scanfile", "simplecov", "Snapfile", "thor", "Thorfile", "Vagrantfile"]
- Cargo Build Results -> []
- Rust -> ["rs"]
- SQL -> ["sql", "ddl", "dml"]
- Scala -> ["scala", "sbt"]
- Shell Script (Bash) -> ["sh", "bash", "zsh", ".bash_aliases", ".bash_functions", ".bash_login", ".bash_logout", ".bash_profile", ".bash_variables", ".bashrc", ".profile", ".textmate_init"]
- HTML (Tcl) -> ["adp"]
- Tcl -> ["tcl"]
- Textile -> ["textile"]
- XML -> ["xml", "xsd", "xslt", "tld", "dtml", "rss", "opml", "svg"]
- YAML -> ["yaml", "yml", "sublime-syntax"]
- Generic Config -> ["cfg", "conf", "config", "ini", "pro"]
- Linker Script -> ["ld"]
- TOML -> ["toml", "tml"]
- TypeScript -> ["ts"]
- TypeScriptReact -> ["tsx"]
- VimL -> ["vim"]
```

If you want to highlight a language not on that list, please open an issue or a pull request on the [Gutenberg repo](https://github.com/Keats/gutenberg).
