use libs::tera::{from_value, to_value, Function as TeraFn, Result, Value};

use libs::chrono::prelude::*;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Now {
    local: DateTime<Local>,
    utc: DateTime<Utc>,
}

impl Now {
    pub fn new(local: DateTime<Local>, utc: DateTime<Utc>) -> Self {
        Self { local, utc }
    }
}

impl TeraFn for Now {
    fn call(&self, args: &HashMap<String, Value>) -> Result<Value> {
        let use_utc = match args.get("utc") {
            Some(val) => match from_value::<bool>(val.clone()) {
                Ok(v) => v,
                Err(_) => {
                    return Err(format!(
                        "Function `now` received utc={val} but `utc` can only be a boolean"
                    )
                    .into());
                }
            },
            None => false,
        };
        let timestamp = match args.get("timestamp") {
            Some(val) => match from_value::<bool>(val.clone()) {
                Ok(v) => v,
                Err(_) => {
                    return Err(format!(
                            "Function `now` received timestamp={val} but `timestamp` can only be a boolean"
                    )
                    .into());
                }
            },
            None => false,
        };

        if use_utc {
            let datetime = self.utc;
            if timestamp {
                return Ok(to_value(datetime.timestamp()).unwrap());
            }
            Ok(to_value(datetime.to_rfc3339()).unwrap())
        } else {
            let datetime = self.local;
            if timestamp {
                return Ok(to_value(datetime.timestamp()).unwrap());
            }
            Ok(to_value(datetime.to_rfc3339()).unwrap())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn now_default() {
        let local = Local::now();
        let utc = local.to_utc();
        let static_fn = Now::new(local, utc);
        let args = HashMap::new();

        let res = static_fn.call(&args).unwrap();
        assert!(res.is_string());
        assert!(res.as_str().unwrap().contains('T'));
    }

    #[test]
    fn now_datetime_utc() {
        let local = Local::now();
        let utc = local.to_utc();
        let static_fn = Now::new(local, utc);
        let mut args = HashMap::new();
        args.insert("utc".to_string(), to_value(true).unwrap());

        let res = static_fn.call(&args).unwrap();
        assert!(res.is_string());
        let val = res.as_str().unwrap();
        assert!(val.contains('T'));
        assert!(val.contains("+00:00"));
    }

    #[test]
    fn now_timestamp() {
        let local = Local::now();
        let utc = local.to_utc();
        let static_fn = Now::new(local, utc);
        let mut args = HashMap::new();
        args.insert("timestamp".to_string(), to_value(true).unwrap());

        let res = static_fn.call(&args).unwrap();
        assert!(res.is_number());
    }
}
