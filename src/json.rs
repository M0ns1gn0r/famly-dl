use serde_json::Value;

pub fn parse_string(v: &Value, property_name: &str) -> Result<String, String> {
    if let Some(str) = v[property_name].as_str() {
        Ok(str.to_string())
    } else {
        Err(format!("'{0}' is not a string", property_name))
    }
}