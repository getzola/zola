use tera;


error_chain! {
    links {
        Tera(tera::Error, tera::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
    }

    errors {
        InvalidConfig {
            description("invalid config")
            display("The config.toml is invalid or is using the wrong type for an argument")
        }
    }
}
