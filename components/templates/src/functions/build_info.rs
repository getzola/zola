use jiff::Zoned;
use jiff::tz::TimeZone;
use tera::{Function, Kwargs, State, TeraResult, Value};

#[derive(Debug, Clone)]
pub struct Now {
    zoned: Zoned,
}

impl Now {
    pub fn new(zoned: Zoned) -> Self {
        Self { zoned }
    }
}

impl Function<TeraResult<Value>> for Now {
    fn call(&self, kwargs: Kwargs, _state: &State) -> TeraResult<Value> {
        let use_utc: bool = kwargs.get("utc")?.unwrap_or(false);
        let timestamp: bool = kwargs.get("timestamp")?.unwrap_or(false);

        let datetime = if use_utc {
            self.zoned.clone().with_time_zone(TimeZone::UTC)
        } else {
            self.zoned.clone()
        };

        if timestamp {
            Ok(Value::from(datetime.timestamp().as_second()))
        } else {
            Ok(Value::from(datetime.timestamp().to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use tera::{Context, Map};

    #[test]
    fn now_default() {
        let now_fn = Now::new(Zoned::now());
        let kwargs = Kwargs::default();
        let ctx = Context::new();

        let res = now_fn.call(kwargs, &State::new(&ctx)).unwrap();
        assert!(res.is_string());
        // jiff timestamp to_string produces ISO 8601 format with T separator
        assert!(res.as_str().unwrap().contains('T'));
    }

    #[test]
    fn now_datetime_utc() {
        let now_fn = Now::new(Zoned::now());

        let mut map = Map::new();
        map.insert("utc".into(), Value::from(true));
        let kwargs = Kwargs::new(Arc::new(map));
        let ctx = Context::new();

        let res = now_fn.call(kwargs, &State::new(&ctx)).unwrap();
        assert!(res.is_string());
        let val = res.as_str().unwrap();
        assert!(val.contains('T'));
        assert!(val.contains("Z"));
    }

    #[test]
    fn now_timestamp() {
        let now_fn = Now::new(Zoned::now());

        let mut map = Map::new();
        map.insert("timestamp".into(), Value::from(true));
        let kwargs = Kwargs::new(Arc::new(map));
        let ctx = Context::new();

        let res = now_fn.call(kwargs, &State::new(&ctx)).unwrap();
        assert!(res.is_number());
    }
}
