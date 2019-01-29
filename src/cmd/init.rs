use std::fs::{canonicalize, create_dir};
use std::path::Path;

use errors::Result;
use utils::fs::create_file;

use console;
use prompt::{ask_bool, ask_url};

const CONFIG: &str = r#"
# The URL the site will be built for
base_url = "%BASE_URL%"

# Whether to automatically compile all Sass files in the sass directory
compile_sass = %COMPILE_SASS%

# Whether to do syntax highlighting
# Theme can be customised by setting the `highlight_theme` variable to a theme supported by Zola
highlight_code = %HIGHLIGHT%

# Whether to build a search index to be used later on by a JavaScript library
build_search_index = %SEARCH%

[extra]
# Put all your custom variables here
"#;

pub fn create_new_project(name: &str) -> Result<()> {
    let path = Path::new(name);
    // Better error message than the rust default
    if path.exists() && path.is_dir() {
        bail!("Folder `{}` already exists", path.to_string_lossy().to_string());
    }

    create_dir(path)?;
    console::info("Welcome to Zola!");

    let base_url = ask_url("> What is the URL of your site?", "https://example.com")?;
    let compile_sass = ask_bool("> Do you want to enable Sass compilation?", true)?;
    let highlight = ask_bool("> Do you want to enable syntax highlighting?", false)?;
    let search = ask_bool("> Do you want to build a search index of the content?", false)?;

    let config = CONFIG
        .trim_start()
        .replace("%BASE_URL%", &base_url)
        .replace("%COMPILE_SASS%", &format!("{}", compile_sass))
        .replace("%SEARCH%", &format!("{}", search))
        .replace("%HIGHLIGHT%", &format!("{}", highlight));

    create_file(&path.join("config.toml"), &config)?;

    create_dir(path.join("content"))?;
    create_dir(path.join("templates"))?;
    create_dir(path.join("static"))?;
    create_dir(path.join("themes"))?;
    if compile_sass {
        create_dir(path.join("sass"))?;
    }

    println!();
    console::success(&format!("Done! Your site was created in {:?}", canonicalize(path).unwrap()));
    println!();
    console::info(
        "Get started by moving into the directory and using the built-in server: `zola serve`",
    );
    println!("Visit https://www.getzola.org for the full documentation.");
    Ok(())
}
