use chrono::{DateTime, Utc};
use error_chain::error_chain;
use serde_json::Value;

use crate::json::parse_string;

error_chain! {
    foreign_links {
        Json(serde_json::Error);
    }
}

pub struct Photo {
    pub id: String,
    /// URL of the *full* size image (valid only for some time).
    pub url: String,
    /// True if the photo is tagged with the target child.
    pub is_tagged: bool,
}

pub struct Comment {
    pub date: DateTime<Utc>,
    pub author: String,
    pub text: String,
}

pub struct FeedItem {
    //pub date: DateTime<Utc>,
    //pub author: String,
    pub text: String,
    //pub photos: Vec<Photo>,
    //pub comments: Vec<Comment>,
}

impl FeedItem {
    // TODO: try to implement a trait instead.

    pub fn from_json(json: &Value) -> Result<FeedItem> {
        let f = FeedItem {
            text: parse_string(json, "body")?,
            // full_name_with_institution: parse_string(json, "name")?,
            // institution: parse_string(&json["institution"], "title")?,
        };
        Ok(f)
    }
}

/// Converts the raw JSON string to a tuple of:
/// * collection of read deed items
/// * an option that is `None` if no more items can be fetched, otherwise `Some` with
/// a *last_item_date* string for a subsequent fetch of feed items
pub fn from_json(json: String) -> Result<(Vec<FeedItem>, Option<String>)> {
    let v: Value = serde_json::from_str(&json)?;
    let feed_items = v["feedItems"].as_array().ok_or("No feedItems array in json")?;
    
    let mut items = vec![];
    for f in feed_items {
        let class = &f["systemPostTypeClass"];
        if !class.is_null() && class.as_str().unwrap().starts_with("Daycare.") {
            // Meta post.
            continue;
        }
        let body = &f["body"];
        if body.is_null() || body.as_str() == Some("") {
            // Invitation posts have an empty body.
            continue;
        }

        let f = FeedItem::from_json(f)
            .expect("Failed to deserialize FeedItem");

        // TODO: ensure has at least one photo tagged with the target childId.

        items.push(f);
    }

    let last_item_date = feed_items.last().map(|x| x["createdDate"].as_str().unwrap().to_string());

    Ok((items, last_item_date))
}