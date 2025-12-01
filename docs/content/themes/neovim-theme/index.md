
+++
title = "neovim"
description = "A only keyboard theme with tabs and file browser"
template = "theme.html"
date = 2025-11-28T20:31:08+01:00

[taxonomies]
theme-tags = []

[extra]
created = 2025-11-28T20:31:08+01:00
updated = 2025-11-28T20:31:08+01:00
repository = "https://github.com/Super-Botman/neovim-theme.git"
homepage = "https://github.com/super-botman/zola-theme"
minimum_version = "0.4"
license = "MIT"
demo = "https://super-botman.github.io"

[extra.author]
name = "0xb0tm4n"
homepage = "https://super-botman.github.io"
+++        

# Neovim like theme 

Neovim theme is a neovim like theme for zola.

![image](https://github.com/user-attachments/assets/0317c951-4975-4150-ac43-7faf4c57aa8b)

exemple: [https://super-botman.github.io](https://super-botman.github.io)

## Installation
```bash
cd themes
git clone https://github.com/Super-Botman/neovim-theme.git
mv neovim-theme/content/readme.md ../content
```

then enable it in your config

```toml
theme = "neovim-theme"
```

## Configuration

```toml
[extra]
######################
# Mandatory settings #
######################

# set the name of the blog
blog_name = "name"

#####################
# Optional settings #
#####################

# set the background image u want
background_image = "/assets/background.jpg"

# this parameter allow you to configure specific init functions/shortcuts and commands
# the value has to be the path of you're config.js file
config_js = "config.js"

# allow you to include custom css into u're blog
custom_css = "style.css"
```

```js
const keys = {
  // "normal" keys are just keys typed on the page
  // for exemple " " is when space is typed
  normal: {
    " ": (event, element) => {
      alert("u pressed space key");
    },
  },

  // this is for keys when shift is pressed
  shortcut: {},
};

const commands = {
  // the key is used to specify the name of the command
  test: (command) => {
    alert("you entered 'test' command");

    // and then the return value with type and message
    return {
      type: "success", // "success" = green text, "error" = red text
      message: "command executed", // the text to show in the command line
    };
  },
};

function custom_init() {
  // here some code
}
```

        