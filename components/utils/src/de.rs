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
    let re = Regex::new(r#"^"?(?P<year>[0-9]{4})-(?P<month>[0-9][0-9]?)-(?P<day>[0-9][0-9]?)(?:(?:[Tt]|[ \t]+)(?P<hour>[0-9][0-9]?):(?P<minute>[0-9]{2}):(?P<second>[0-9]{2})(?P<fraction>\.[0-9]{0,9})?[ \t]*(?:(?P<utc>Z)|(?P<offset>(?P<offset_hour>[-+][0-9][0-9]?)(?::(?P<offset_minute>[0-9][0-9]))?))?)?"?$"#).unwrap();
    let captures = if let Some(captures_) = re.captures(date_string) {
        Ok(captures_)
    } else {
        Err(anyhow!("Error parsing YAML datetime"))
    }?;
    let year = captures.name("year").unwrap().as_str();
    let month = captures.name("month").unwrap().as_str();
    let day = captures.name("day").unwrap().as_str();
    let hour = if let Some(hour_) = captures.name("hour") { hour_.as_str() } else { "0" };
    let minute = if let Some(minute_) = captures.name("minute") { minute_.as_str() } else { "0" };
    let second = if let Some(second_) = captures.name("second") { second_.as_str() } else { "0" };
    let fraction_raw =
        if let Some(fraction_) = captures.name("fraction") { fraction_.as_str() } else { "" };
    let fraction_intermediate = fraction_raw.trim_end_matches("0");
    //
    // Prepare for eventual conversion into nanoseconds
    let fraction = if fraction_intermediate.len() > 0 { fraction_intermediate } else { "0" };
    let maybe_timezone_hour = captures.name("offset_hour");
    let maybe_timezone_minute = captures.name("offset_minute");

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
        .replace_hour(hour.parse().unwrap())?
        .replace_minute(minute.parse().unwrap())?
        .replace_second(second.parse().unwrap())?
        .replace_nanosecond((fraction.parse::<f64>().unwrap_or(0.0) * 1_000_000_000.0) as u32)?)
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
    fn yaml_draft_timestamp_pass() {
        // tests only the values from the YAML 1.1 Timestamp Draft
        // See https://yaml.org/type/timestamp.html
        let canonical = "2001-12-15T02:59:43.1Z";
        let valid_iso8601 = "2001-12-14t21:59:43.10-05:00";
        let space_separated = "2001-12-14 21:59:43.10 -5";
        let no_time_zone = "2001-12-15 2:59:43.10";
        let date = "2002-12-14";
        assert_eq!(
            parse_yaml_datetime(canonical).unwrap(),
            datetime!(2001-12-15 02:59:43.100 +00:00)
        );
        assert_eq!(
            parse_yaml_datetime(valid_iso8601).unwrap(),
            datetime!(2001-12-14 21:59:43.100 -05:00)
        );
        assert_eq!(
            parse_yaml_datetime(space_separated).unwrap(),
            datetime!(2001-12-14 21:59:43.100 -05:00)
        );
        assert_eq!(
            parse_yaml_datetime(no_time_zone).unwrap(),
            datetime!(2001-12-15 02:59:43.100 +00:00)
        );
        assert_eq!(parse_yaml_datetime(date).unwrap(), datetime!(2002-12-14 00:00:00.000 +00:00));
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

    #[test]
    fn toml_test_pass() {
        // tests subset from toml-test
        // Taken from https://github.com/toml-lang/toml-test/tree/a80ce8268cbcf5ea95f02b2e6d6cc38406ce28c9/tests/valid/datetime
        let space = "1987-07-05 17:45:00Z";
        // Z is not allowed to be lowercase
        let lower = "1987-07-05t17:45:00Z";

        let first_offset = "0001-01-01 00:00:00Z";
        let first_local = "0001-01-01 00:00:00";
        let first_date = "0001-01-01";
        let last_offset = "9999-12-31 23:59:59Z";
        let last_local = "9999-12-31 23:59:59";
        let last_date = "9999-12-31";

        // valid leap years
        let datetime_2000 = "2000-02-29 15:15:15Z";
        let datetime_2024 = "2024-02-29 15:15:15Z";

        // milliseconds
        let ms1 = "1987-07-05T17:45:56.123Z";
        let ms2 = "1987-07-05T17:45:56.6Z";

        // timezones
        let utc = "1987-07-05T17:45:56Z";
        let pdt = "1987-07-05T17:45:56-05:00";
        let nzst = "1987-07-05T17:45:56+12:00";
        let nzdt = "1987-07-05T17:45:56+13:00"; // DST

        assert_eq!(parse_yaml_datetime(space).unwrap(), datetime!(1987-07-05 17:45:00.000 +00:00));
        assert_eq!(parse_yaml_datetime(lower).unwrap(), datetime!(1987-07-05 17:45:00.000 +00:00));

        assert_eq!(
            parse_yaml_datetime(first_offset).unwrap(),
            datetime!(0001-01-01 00:00:00.000 +00:00)
        );
        assert_eq!(
            parse_yaml_datetime(first_local).unwrap(),
            datetime!(0001-01-01 00:00:00.000 +00:00)
        );
        assert_eq!(
            parse_yaml_datetime(first_date).unwrap(),
            datetime!(0001-01-01 00:00:00.000 +00:00)
        );
        assert_eq!(
            parse_yaml_datetime(last_offset).unwrap(),
            datetime!(9999-12-31 23:59:59.000 +00:00)
        );
        assert_eq!(
            parse_yaml_datetime(last_local).unwrap(),
            datetime!(9999-12-31 23:59:59.000 +00:00)
        );
        assert_eq!(
            parse_yaml_datetime(last_date).unwrap(),
            datetime!(9999-12-31 00:00:00.000 +00:00)
        );

        assert_eq!(
            parse_yaml_datetime(datetime_2000).unwrap(),
            datetime!(2000-02-29 15:15:15.000 +00:00)
        );
        assert_eq!(
            parse_yaml_datetime(datetime_2024).unwrap(),
            datetime!(2024-02-29 15:15:15.000 +00:00)
        );

        assert_eq!(parse_yaml_datetime(ms1).unwrap(), datetime!(1987-07-05 17:45:56.123 +00:00));
        assert_eq!(parse_yaml_datetime(ms2).unwrap(), datetime!(1987-07-05 17:45:56.600 +00:00));

        assert_eq!(parse_yaml_datetime(utc).unwrap(), datetime!(1987-07-05 17:45:56.000 +00:00));
        assert_eq!(parse_yaml_datetime(pdt).unwrap(), datetime!(1987-07-05 22:45:56.000 +00:00));
        assert_eq!(parse_yaml_datetime(nzst).unwrap(), datetime!(1987-07-05 05:45:56.000 +00:00));
        assert_eq!(parse_yaml_datetime(nzdt).unwrap(), datetime!(1987-07-05 04:45:56.000 +00:00));
    }

    #[test]
    fn toml_test_fail() {
        let not_a_leap_year = "2100-02-29T15:15:15Z";
        assert!(parse_yaml_datetime(not_a_leap_year).is_err());

        let feb_30 = "1988-02-30T15:15:15Z";
        assert!(parse_yaml_datetime(feb_30).is_err());

        let hour_over = "2006-01-01T24:00:00-00:00";
        assert!(parse_yaml_datetime(hour_over).is_err());

        let mday_over = "2006-01-32T00:00:00-00:00";
        assert!(parse_yaml_datetime(mday_over).is_err());

        let mday_under = "2006-01-00T00:00:00-00:00";
        assert!(parse_yaml_datetime(mday_under).is_err());

        let minute_over = "2006-01-01T00:60:00-00:00";
        assert!(parse_yaml_datetime(minute_over).is_err());

        let month_over = "2006-13-01T00:00:00-00:00";
        assert!(parse_yaml_datetime(month_over).is_err());

        let month_under = "2007-00-01T00:00:00-00:00";
        assert!(parse_yaml_datetime(month_under).is_err());

        let no_secs = "1987-07-05T17:45Z";
        assert!(parse_yaml_datetime(no_secs).is_err());

        let no_sep = "1987-07-0517:45:00Z";
        assert!(parse_yaml_datetime(no_sep).is_err());

        // 'time' supports up until ±25:59:59
        let offset_overflow = "1985-06-18 17:04:07+26:00";
        assert!(parse_yaml_datetime(offset_overflow).is_err());

        let offset_overflow = "1985-06-18 17:04:07+12:61";
        assert!(parse_yaml_datetime(offset_overflow).is_err());

        let second_overflow = "2006-01-01T00:00:61-00:00";
        assert!(parse_yaml_datetime(second_overflow).is_err());

        let y10k = "10000-01-01 00:00:00z";
        assert!(parse_yaml_datetime(y10k).is_err());
    }
}
