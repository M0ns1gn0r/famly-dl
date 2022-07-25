use serde_json::Value;
use error_chain::error_chain;

error_chain! {
    foreign_links {
        Json(serde_json::Error);
    }
}

pub struct ChildInfo {
    pub id: String,
    pub full_name_with_institution: String,
    pub institution: String,
}

impl ChildInfo {
    /// Parses the full name to return the first name only, if possible.
    /// Otherwise falls back to the full name.
    pub fn get_first_name(&self) -> String {
        if let Some(first_part) = self.full_name_with_institution.split_whitespace().next() {
            first_part.to_string()
        } else {
            self.full_name_with_institution.clone()
        }        
    }
}

fn parse_string(v: &Value, property_name: &str) -> Result<String> {
    let str = v[property_name]
        .as_str()
        .ok_or(format!("'{0}' is not a string", property_name))?;
    Ok(str.to_string())
}

pub fn from_json(json: String) -> Result<Vec<ChildInfo>> {
    let v: Value = serde_json::from_str(&json)?;
    let children = v["children"].as_array().ok_or("No children array in json")?;
    
    let mut res = vec!();
    for c in children {
        res.push(ChildInfo {
            id: parse_string(c, "childId")?,
            full_name_with_institution: parse_string(c, "name")?,
            institution: parse_string(&c["institution"], "title")?,
        })        
    }
    Ok(res)
}