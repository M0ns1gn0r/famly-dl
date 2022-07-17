use serde_json::Value;
use error_chain::error_chain;

error_chain! {
    foreign_links {
        Json(serde_json::Error);
    }
}

pub struct ChildInfo {
    pub id: String,
    pub name: String,
    pub institution: String,
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
            name: parse_string(c, "name")?,
            institution: parse_string(&c["institution"], "title")?,
        })        
    }
    Ok(res)
}