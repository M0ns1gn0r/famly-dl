use std::fmt::format;

use chrono::{DateTime, Utc};
use error_chain::error_chain;
use serde_json::Value;

use crate::json::*;

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

impl TryFrom<&Value> for Comment {
    type Error = String;

    fn try_from(json: &Value) -> core::result::Result<Self, Self::Error> {
        let name = parse_string(&json["sender"], "name")?;
        let child_name = parse_string(&json["sender"], "subtitle");

        let author = if let Ok(child_name) = child_name {
            if !child_name.is_empty() {
                format!("{0} [{1}]", name, child_name)
            } else {
                name
            }
        } else {
            name
        };

        let c = Comment {
            date: parse_date(json, "createdDate")?,
            text: parse_string(json, "body")?,
            author,
        };
        Ok(c)
    }
}

pub struct Post {
    // Famly doesn't store time zones, all dates are in UTC anyways.
    pub date: DateTime<Utc>,
    pub author: String,
    pub text: String,
    //pub photos: Vec<Photo>,
    pub comments: Vec<Comment>,
}

impl TryFrom<&Value> for Post {
    type Error = String;

    fn try_from(json: &Value) -> core::result::Result<Self, Self::Error> {
        let untyped_comments = json["comments"].as_array().ok_or("No comments array in post json")?;

        let p = Post {
            date: parse_date(json, "createdDate")?,
            text: parse_string(json, "body")?,
            author: parse_string(&json["sender"], "name")?,
            comments: untyped_comments
                .iter()
                .map(|c| c.try_into().expect("Failed to deserialize a comment json"))
                .collect(),
        };
        Ok(p)
    }
}

/// Converts the raw JSON string to a tuple of:
/// * collection of posts
/// * an option value: `None` if there was no feed items in the json, otherwise `Some` with
/// the *last_item_date* string for fetching of subsequent feed items.
pub fn from_feed_json(feed_json: String) -> Result<(Vec<Post>, Option<String>)> {
    let parsed_json: Value = serde_json::from_str(&feed_json)?;
    let feed_items = parsed_json["feedItems"].as_array().ok_or("No feedItems array in json")?;
    
    let mut posts = vec![];
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

        let f = f.try_into().expect("Failed to deserialize a feed item json to a post");

        // TODO: ensure has at least one photo tagged with the target childId.

        posts.push(f);
    }

    let last_item_date = feed_items.last().map(|x| x["createdDate"].as_str().unwrap().to_string());

    Ok((posts, last_item_date))
}