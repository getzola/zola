+++
title = "Syntax Highlighting"
weight = 80
+++

Zola comes with built-in syntax highlighting but you first
need to enable it in the [configuration](@/documentation/getting-started/configuration.md).

Once this is done, Zola will automatically highlight all code blocks
in your content. A code block in Markdown looks like the following:

````md

```rust
let highlight = true;
```

````

You can replace the `rust` by the language you want to highlight or not put anything to get it
interpreted as plain text.

Here is a full list of the supported languages and the short names you can use:

```
- Plain Text -> ["txt"]
- Assembly x86 (NASM) -> ["asm", "inc", "nasm"]
- Crystal -> ["cr"]
- Dart -> ["dart"]
- Elixir -> ["ex", "exs"]
- fsharp -> ["fs"]
- Handlebars -> ["handlebars", "handlebars.html", "hbr", "hbrs", "hbs", "hdbs", "hjs", "mu", "mustache", "rac", "stache", "template", "tmpl"]
- Jinja2 -> ["j2", "jinja2"]
- Julia -> ["jl"]
- Kotlin -> ["kt", "kts"]
- Less -> ["less", "css.less"]
- MiniZinc (MZN) -> ["mzn", "dzn"]
- Nim -> ["nim", "nims"]
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
- Git Attributes -> ["attributes", "gitattributes", ".gitattributes"]
- Git Commit -> ["COMMIT_EDITMSG", "MERGE_MSG", "TAG_EDITMSG"]
- Git Config -> ["gitconfig", ".gitconfig", ".gitmodules"]
- Git Ignore -> ["exclude", "gitignore", ".gitignore"]
- Git Link -> [".git"]
- Git Log -> ["gitlog"]
- Git Rebase Todo -> ["git-rebase-todo"]
- Go -> ["go"]
- Graphviz (DOT) -> ["dot", "DOT", "gv"]
- Groovy -> ["groovy", "gvy", "gradle", "Jenkinsfile"]
- HTML -> ["html", "htm", "shtml", "xhtml"]
- Haskell -> ["hs"]
- Literate Haskell -> ["lhs"]
- Java Server Page (JSP) -> ["jsp"]
- Java -> ["java", "bsh"]
- Java Properties -> ["properties"]
- JSON -> ["json", "sublime-settings", "sublime-menu", "sublime-keymap", "sublime-mousemap", "sublime-theme", "sublime-build", "sublime-project", "sublime-completions", "sublime-commands", "sublime-macro", "sublime-color-scheme"]
- JavaScript -> ["js", "htc"]
- BibTeX -> ["bib"]
- LaTeX -> ["tex", "ltx"]
- TeX -> ["sty", "cls"]
- Lisp -> ["lisp", "cl", "clisp", "l", "mud", "el", "scm", "ss", "lsp", "fasl"]
- Lua -> ["lua"]
- Makefile -> ["make", "GNUmakefile", "makefile", "Makefile", "makefile.am", "Makefile.am", "makefile.in", "Makefile.in", "OCamlMakefile", "mak", "mk"]
- Markdown -> ["md", "mdown", "markdown", "markdn"]
- MATLAB -> ["matlab"]
- OCaml -> ["ml", "mli"]
- OCamllex -> ["mll"]
- OCamlyacc -> ["mly"]
- Objective-C++ -> ["mm", "M", "h"]
- Objective-C -> ["m", "h"]
- PHP -> ["php", "php3", "php4", "php5", "php7", "phps", "phpt", "phtml"]
- Pascal -> ["pas", "p", "dpr"]
- Perl -> ["pl", "pm", "pod", "t", "PL"]
- Python -> ["py", "py3", "pyw", "pyi", "pyx", "pyx.in", "pxd", "pxd.in", "pxi", "pxi.in", "rpy", "cpy", "SConstruct", "Sconstruct", "sconstruct", "SConscript", "gyp", "gypi", "Snakefile", "wscript"]
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
- Rust -> ["rs"]
- SQL -> ["sql", "ddl", "dml"]
- Scala -> ["scala", "sbt"]
- Bourne Again Shell (bash) -> ["sh", "bash", "zsh", "fish", ".bash_aliases", ".bash_completions", ".bash_functions", ".bash_login", ".bash_logout", ".bash_profile", ".bash_variables", ".bashrc", ".profile", ".textmate_init", ".zshrc"]
- HTML (Tcl) -> ["adp"]
- Tcl -> ["tcl"]
- Textile -> ["textile"]
- XML -> ["xml", "xsd", "xslt", "tld", "dtml", "rss", "opml", "svg"]
- YAML -> ["yaml", "yml", "sublime-syntax"]
- PowerShell -> ["ps1", "psm1", "psd1"]
- SWI-Prolog -> ["pro"]
- Reason -> ["re", "rei"]
- CMake C Header -> ["h.in"]
- CMake C++ Header -> ["hh.in", "hpp.in", "hxx.in", "h++.in"]
- CMake -> ["CMakeLists.txt", "cmake"]
- CMakeCache -> ["CMakeCache.txt"]
- Generic Config -> ["cfg", "conf", "config", "ini", "pro", "mak", "mk", "Doxyfile", "inputrc", ".inputrc", "dircolors", ".dircolors", "gitmodules", ".gitmodules", "gitignore", ".gitignore", "gitattributes", ".gitattributes"]
- Elm -> ["elm"]
- Linker Script -> ["ld"]
- Swift -> ["swift"]
- TOML -> ["toml", "tml"]
- TypeScript -> ["ts"]
- TypeScriptReact -> ["tsx"]
- VimL -> ["vim"]
- Nix -> ["nix"]
- TOML -> ["toml", "tml", "Cargo.lock", "Gopkg.lock", "Pipfile"]
```

If you want to highlight a language not on that list, please open an issue or a pull request on the [Zola repo](https://github.com/getzola/zola).
Alternatively, the `extra_syntaxes` config option can be used to add additional syntax files.

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

you would set your `extra_syntaxes` to `["syntaxes", "syntaxes/Sublime-Language1"]` in order to load `lang1.sublime-syntax` and `lang2.sublime-syntax`.
