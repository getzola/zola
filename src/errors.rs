
error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        FolderExists(name: String) {
            description("folder already exists")
            display("Folder '{}' already exists.", name)
        }
    }
}
