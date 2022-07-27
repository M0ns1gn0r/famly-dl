use chrono::{DateTime, Utc};
use serde_json::Value;

pub fn parse_int(v: &Value, property_name: &str) -> Result<u64, String> {
    if let Some(val) = v[property_name].as_u64() {
        Ok(val)
    } else {
        Err(format!("'{0}' is not an unsigned integer", property_name))
    }
}

pub fn parse_string(v: &Value, property_name: &str) -> Result<String, String> {
    if let Some(str) = v[property_name].as_str() {
        Ok(str.to_string())
    } else {
        Err(format!("'{0}' is not a string", property_name))
    }
}

pub fn parse_date(v: &Value, property_name: &str) -> Result<DateTime<Utc>, String> {
    if let Some(str) = v[property_name].as_str() {
        let dt = DateTime::parse_from_rfc3339(str)
            .map_err(|e| format!(
                "Failed to parse '{0}' with value '{1}' as date: {2}",
                property_name, str, e))
            .map(|d| d.with_timezone(&Utc));
        dt
    } else {
        Err(format!("'{0}' is not a string", property_name))
    }
}