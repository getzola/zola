use tera;
use toml;

error_chain! {
    errors {}

    links {
        Tera(tera::Error, tera::ErrorKind);
    }

    foreign_links {
        Io(::std::io::Error);
        Ffi(::std::ffi::NulError);
        SystemTimeError(::std::time::SystemTimeError);
        Toml(toml::de::Error);
    }
}
