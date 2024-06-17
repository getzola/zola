use core::convert::TryFrom;
use errors::{anyhow, Result};
use libs::regex::Regex;
use libs::tera::{Map, Value};
use libs::time;
use libs::time::format_description::well_known::Rfc3339;
use libs::toml;
use serde::{Deserialize, Deserializer};

pub fn parse_yaml_datetime(date_string: &str) -> Result<time::OffsetDateTime> {
    // See https://github.com/getzola/zola/issues/2071#issuecomment-1530610650
    let re = Regex::new(r#"^"?([0-9]{4})-([0-9][0-9]?)-([0-9][0-9]?)([Tt]|[ \t]+)([0-9][0-9]?):([0-9]{2}):([0-9]{2})\.([0-9]*)?Z?([ \t]([-+][0-9][0-9]?)(:([0-9][0-9]?))?Z?|([-+][0-9]{2})?:([0-9]{2})?)?|([0-9]{4})-([0-9]{2})-([0-9]{2})"?$"#).unwrap();
    let captures = if let Some(captures_) = re.captures(date_string) {
        Ok(captures_)
    } else {
        Err(anyhow!("Error parsing YAML datetime"))
    }?;
    let year =
        if let Some(cap) = captures.get(1) { cap } else { captures.get(15).unwrap() }.as_str();
    let month =
        if let Some(cap) = captures.get(2) { cap } else { captures.get(16).unwrap() }.as_str();
    let day =
        if let Some(cap) = captures.get(3) { cap } else { captures.get(17).unwrap() }.as_str();
    let hours = if let Some(hours_) = captures.get(5) { hours_.as_str() } else { "0" };
    let minutes = if let Some(minutes_) = captures.get(6) { minutes_.as_str() } else { "0" };
    let seconds = if let Some(seconds_) = captures.get(7) { seconds_.as_str() } else { "0" };
    let fractional_seconds_raw =
        if let Some(fractionals) = captures.get(8) { fractionals.as_str() } else { "" };
    let fractional_seconds_intermediate = fractional_seconds_raw.trim_end_matches("0");
    //
    // Prepare for eventual conversion into nanoseconds
    let fractional_seconds = if fractional_seconds_intermediate.len() > 0
        && fractional_seconds_intermediate.len() <= 9
    {
        fractional_seconds_intermediate
    } else {
        "0"
    };
    let maybe_timezone_hour_1 = captures.get(10);
    let maybe_timezone_minute_1 = captures.get(12);
    let maybe_timezone_hour_2 = captures.get(13);
    let maybe_timezone_minute_2 = captures.get(14);
    let maybe_timezone_hour;
    let maybe_timezone_minute;
    if maybe_timezone_hour_2.is_some() {
        maybe_timezone_hour = maybe_timezone_hour_2;
        maybe_timezone_minute = maybe_timezone_minute_2;
    } else {
        maybe_timezone_hour = maybe_timezone_hour_1;
        maybe_timezone_minute = maybe_timezone_minute_1;
    }

    let mut offset_datetime = time::OffsetDateTime::UNIX_EPOCH;

    if let Some(hour) = maybe_timezone_hour {
        let minute_str =
            if let Some(minute_) = maybe_timezone_minute { minute_.as_str() } else { "0" };
        offset_datetime = offset_datetime.to_offset(time::UtcOffset::from_hms(
            hour.as_str().parse()?,
            minute_str.parse()?,
            0,
        )?);
    }

    // Free parse unwraps since we know everything is a digit courtesy of prior regex.
    Ok(offset_datetime
        .replace_year(year.parse().unwrap())?
        .replace_month(time::Month::try_from(month.parse::<u8>().unwrap())?)?
        .replace_day(day.parse().unwrap())?
        .replace_hour(hours.parse().unwrap())?
        .replace_minute(minutes.parse().unwrap())?
        .replace_second(seconds.parse().unwrap())?
        .replace_nanosecond(fractional_seconds.parse::<u32>().unwrap() * 100_000_000)?)
}

/// Used as an attribute when we want to convert from TOML to a string date
/// If a TOML datetime isn't present, it will accept a string and push it through
/// TOML's date time parser to ensure only valid dates are accepted.
/// Inspired by this proposal: <https://github.com/alexcrichton/toml-rs/issues/269>
pub fn from_unknown_datetime<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;
    use std::str::FromStr;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum MaybeDatetime {
        Datetime(toml::value::Datetime),
        String(String),
    }

    match MaybeDatetime::deserialize(deserializer)? {
        MaybeDatetime::Datetime(d) => Ok(Some(d.to_string())),
        MaybeDatetime::String(s) => {
            if let Ok(d) = toml::value::Datetime::from_str(&s) {
                Ok(Some(d.to_string()))
            } else if let Ok(d) = parse_yaml_datetime(&s) {
                // Ensure that the resulting string is easily reparseable down the line.
                // In content::front_matter::page.rs where these strings are currently used,
                // Rfc3339 works with the explicit demands in that code but not always with the result of
                // _to_string.
                Ok(Some(d.format(&Rfc3339).unwrap()))
            } else {
                Err(D::Error::custom("Unable to parse datetime"))
            }
        }
    }
}

/// Returns key/value for a converted date from TOML.
/// If the table itself is the TOML struct, only return its value without the key
fn convert_toml_date(table: Map<String, Value>) -> Value {
    let mut new = Map::new();

    for (k, v) in table {
        if k == "$__toml_private_datetime" {
            return v;
        }

        match v {
            Value::Object(o) => {
                new.insert(k, convert_toml_date(o));
            }
            _ => {
                new.insert(k, v);
            }
        }
    }

    Value::Object(new)
}

/// TOML datetimes will be serialized as a struct but we want the
/// stringified version for json, otherwise they are going to be weird
pub fn fix_toml_dates(table: Map<String, Value>) -> Value {
    let mut new = Map::new();

    for (key, value) in table {
        match value {
            Value::Object(o) => {
                new.insert(key, convert_toml_date(o));
            }
            Value::Array(arr) => {
                let mut new_arr = Vec::with_capacity(arr.len());
                for v in arr {
                    match v {
                        Value::Object(o) => new_arr.push(fix_toml_dates(o)),
                        _ => new_arr.push(v),
                    };
                }
                new.insert(key, Value::Array(new_arr));
            }
            _ => {
                new.insert(key, value);
            }
        }
    }

    Value::Object(new)
}

#[cfg(test)]
mod tests {
    use super::parse_yaml_datetime;
    use time::macros::datetime;

    #[test]
    fn yaml_spec_examples_pass() {
        let canonical = "2001-12-15T02:59:43.1Z";
        let valid_iso8601 = "2001-12-14t21:59:43.10-05:00";
        let space_separated = "2001-12-14 21:59:43.10 -5";
        let no_time_zone = "2001-12-15 2:59:43.10";
        let date = "2002-12-14";
        assert_eq!(parse_yaml_datetime(canonical).unwrap(), datetime!(2001-12-15 2:59:43.1 +0));
        assert_eq!(
            parse_yaml_datetime(valid_iso8601).unwrap(),
            datetime!(2001-12-14 21:59:43.1 -5)
        );
        assert_eq!(
            parse_yaml_datetime(space_separated).unwrap(),
            datetime!(2001-12-14 21:59:43.1 -5)
        );
        assert_eq!(parse_yaml_datetime(no_time_zone).unwrap(), datetime!(2001-12-15 2:59:43.1 +0));
        assert_eq!(parse_yaml_datetime(date).unwrap(), datetime!(2002-12-14 0:00:00 +0));
    }

    #[test]
    fn yaml_spec_invalid_dates_fail() {
        let invalid_month = "2001-13-15";
        assert!(parse_yaml_datetime(invalid_month).is_err());

        let invalid_month = "2001-13-15T02:59:43.1Z";
        assert!(parse_yaml_datetime(invalid_month).is_err());

        let no_digits_in_year = "xxxx-12-15";
        assert!(parse_yaml_datetime(no_digits_in_year).is_err());

        let no_digits_in_year = "xxxx-12-15T02:59:43.1Z";
        assert!(parse_yaml_datetime(no_digits_in_year).is_err());

        let no_digits_in_month = "2001-xx-15";
        assert!(parse_yaml_datetime(no_digits_in_month).is_err());

        let no_digits_in_month = "2001-xx-15T02:59:43.1Z";
        assert!(parse_yaml_datetime(no_digits_in_month).is_err());

        let no_digits_in_day = "2001-12-xx";
        assert!(parse_yaml_datetime(no_digits_in_day).is_err());

        let no_digits_in_day = "2001-12-xx:59:43.1Z";
        assert!(parse_yaml_datetime(no_digits_in_day).is_err());

        let unparseable_time = "2001-12-15:69:43.1Z";
        assert!(parse_yaml_datetime(unparseable_time).is_err());

        let unparseable_time = "2001-12-15:59:4x.1Z";
        assert!(parse_yaml_datetime(unparseable_time).is_err());
    }
}
