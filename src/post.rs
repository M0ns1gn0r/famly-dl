use chrono::{DateTime, Utc, TimeZone};
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
    pub date: DateTime<Utc>,
    /// URL of the *full* size image (valid only for some time).
    pub url: String,
    tags: Vec<String>,
}

impl Photo {
    /// Returns true if the photo is tagged with the target child.
    pub fn is_tagged(&self, child_id: &String) -> bool {
        self.tags.contains(child_id)
    }

    /// Returns a unique-ish file name that should be used to store this photo.
    pub fn get_file_name(&self) -> String {
        let date = self.date.format("%Y-%m-%d_%H-%M-%S");
        let short_id: String = self.id.chars().take(4).collect();
        format!("{}_{}.jpg", date, short_id)
    }

    /// Converts the raw JSON string to a tuple of:
    /// * collection of photos
    /// * an option value: `None` if there were no items in the json, otherwise `Some` with
    /// the *last_item_date* string for fetching of subsequent items.
    pub fn from_json_array(json: String) -> Result<(Vec<Photo>, Option<String>)> {
        let parsed_json: Value = serde_json::from_str(&json)?;
        let items = parsed_json.as_array().ok_or("No photos array in json")?;

        let photos = items.iter()
            .map(|i| i.try_into().expect("Failed to deserialize a photo json"))
            .collect();
    
        let last_item_date = items.last().map(|x| x["createdAt"].as_str().unwrap().to_string());
    
        Ok((photos, last_item_date))
    }
}

impl TryFrom<&Value> for Photo {
    type Error = String;

    fn try_from(json: &Value) -> core::result::Result<Self, Self::Error> {
        let date = if json["createdAt"].is_object() {
            // V1 version of API.
            let date_val = parse_string(&json["createdAt"], "date")?;
            let date_str = date_val.as_str();
            Utc.datetime_from_str(date_str, "%Y-%m-%d %H:%M:%S%.6f")
                .map_err(|e| format!("Failed to parse '{0}' as date: {1}", date_str, e))?
        } else {
            // V2 version of API.
            parse_date(&json, "createdAt")?
        };

        let prefix = parse_string(&json, "prefix")?;
        let key = parse_string(&json, "key")?;
        let height = parse_int(&json, "height")?;
        let width = parse_int(&json, "width")?;

        let untyped_tags = json["tags"].as_array().ok_or("No tags array in image json")?;
        let tags = untyped_tags
            .iter()
            .map(|t| parse_string(t, "childId").expect("Failed to deserialize a tag json"))
            .collect();

        let p = Photo {
            id: parse_string(json, "imageId")?,
            date,
            url: format!("{0}/{1}x{2}/{3}", prefix, width, height, key),
            tags,
        };
        Ok(p)
    }
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
                format!("{0} | {1}", name, child_name)
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
    pub photos: Vec<Photo>,
    pub comments: Vec<Comment>,
}

impl Post {
    /// Returns the post's title by taking first few symbols of its text.
    pub fn get_title(&self) -> String {
        let mut content_started = false;

        self.text.chars()
            .filter(|x| {
                if *x == ' ' {
                    // Skips leading whitespaces.
                    content_started
                } else if x.is_alphanumeric() {
                    content_started = true;
                    true
                } else {
                    false
                }
            })
            .take(25)
            .collect::<String>()
    }

    /// Converts the raw JSON string to a tuple of:
    /// * collection of posts
    /// * an option value: `None` if there were no feed items in the json, otherwise `Some` with
    /// the *last_item_date* string for fetching of subsequent feed items.
    pub fn from_feed_json(feed_json: String, child_id: &String) -> Result<(Vec<Post>, Option<String>)> {
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

            let post: Post = f.try_into().expect("Failed to deserialize a feed item json to a post");

            if post.photos.iter().all(|p| !p.is_tagged(child_id)) {
                // At least one photo must be tagged with the target child.
                continue;
            }

            posts.push(post);
        }

        let last_item_date = feed_items.last().map(|x| x["createdDate"].as_str().unwrap().to_string());

        Ok((posts, last_item_date))
    }
}

impl TryFrom<&Value> for Post {
    type Error = String;

    fn try_from(json: &Value) -> core::result::Result<Self, Self::Error> {
        let photos = json["images"]
            .as_array().ok_or("No images array in post json")?
            .iter()
            .map(|c| c.try_into().expect("Failed to deserialize an image json"))
            .collect();
        let comments = json["comments"]
            .as_array().ok_or("No comments array in post json")?
            .iter()
            .map(|c| c.try_into().expect("Failed to deserialize a comment json"))
            .collect();

        let p = Post {
            date: parse_date(json, "createdDate")?,
            text: parse_string(json, "body")?,
            author: parse_string(&json["sender"], "name")?,
            photos,
            comments,
        };
        Ok(p)
    }
}