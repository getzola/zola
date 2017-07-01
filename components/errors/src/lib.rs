#[macro_use]
extern crate error_chain;
extern crate tera;
extern crate toml;

error_chain! {
    errors {}

    links {
        Tera(tera::Error, tera::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
        Toml(toml::de::Error);
    }
}

// So we can use bail! in all other crates
#[macro_export]
macro_rules! bail {
    ($e:expr) => {
        return Err($e.into());
    };
    ($fmt:expr, $($arg:tt)+) => {
        return Err(format!($fmt, $($arg)+).into());
    };
}

