use std::fs::{create_dir, canonicalize};
use std::path::Path;

use errors::Result;
use utils::fs::create_file;

use prompt::{ask_bool, ask_url};
use console;


const CONFIG: &'static str = r#"
# The URL the site will be built for
base_url = "%BASE_URL%"

# Whether to automatically compile all Sass files in the sass directory
compile_sass = %COMPILE_SASS%

# Whether to do syntax highlighting
# Theme can be customised by setting the `highlight_theme` variable to a theme supported by Gutenberg
highlight_code = %HIGHLIGHT%

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
    console::info("Welcome to Gutenberg!");

    let base_url = ask_url("> What is the URL of your site?", "https://example.com")?;
    let compile_sass = ask_bool("> Do you want to enable Sass compilation?", true)?;
    let highlight = ask_bool("> Do you want to enable syntax highlighting?", false)?;

    let config = CONFIG
        .trim_left()
        .replace("%BASE_URL%", &base_url)
        .replace("%COMPILE_SASS%", &format!("{}", compile_sass))
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
    console::info("Get started by moving into the directory and using the built-in server: `gutenberg serve`");
    println!("Visit https://www.getgutenberg.io for the full documentation.");
    Ok(())
}
