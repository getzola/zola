
error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        InvalidFrontMatter(name: String) {
            description("frontmatter is invalid")
            display("Front Matter of file '{}' is missing or is invalid", name)
        }
        InvalidConfig {
            description("invalid config")
            display("The config.toml is invalid or is using the wrong type for an argument")
        }
        FolderExists(name: String) {
            description("folder already exists")
            display("Folder '{}' already exists.", name)
        }
    }
}
