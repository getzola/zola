use tera::{Error, Kwargs, State, TeraResult};

pub fn get_env(kwargs: Kwargs, _: &State) -> TeraResult<String> {
    let name: &str = kwargs.must_get("name")?;
    let default: Option<String> = kwargs.get("default")?;
    match std::env::var(name).ok() {
        Some(res) => Ok(res),
        None => {
            if let Some(default) = default {
                Ok(default)
            } else {
                Err(Error::message(format!("Environment variable `{name}` not found")))
            }
        }
    }
}
