mod config;
mod console;
mod child_info;
mod post;
mod file_system;
mod http;
mod html;
mod json;

use child_info::ChildInfo;
use chrono::Datelike;
use config::Config;
use error_chain::error_chain;
use file_system::create_dir;
use post::{Post, Photo};
use reqwest::blocking::Client;

error_chain! {
    links {
        ChildInfo(child_info::Error, child_info::ErrorKind);
        Post(post::Error, post::ErrorKind);
        Http(http::Error, http::ErrorKind);
    }
    foreign_links {
        Io(std::io::Error);
    }
}

fn choose_target_child(child_infos: &Vec<ChildInfo>) -> &ChildInfo {
    let children_count = child_infos.len();

    if children_count < 2 {
        let child = &child_infos[0];
        return child;
    }

    loop {
        if let Some(child_number) =
            console::choose_number("Select the child (CTRL+C to exit): ", children_count) {
            let child = &child_infos[child_number - 1];
            println!("{0} is selected ({1})", child.get_first_name(), child.id);
            return child;
        }

        println!("Invalid number")
    }
}

fn store_posts(posts: &Vec<Post>, child: &ChildInfo) -> Result<()> {
    println!("Storing posts...");

    let name = &child.get_first_name();
    let root_dir = std::path::Path::new(name);
    let tagged_photos_dir = root_dir.join("tagged_photos");
    std::fs::create_dir_all(&tagged_photos_dir)?;

    let total = posts.len();
    let mut i = 0;
    for p in posts {
        let posts_dir = root_dir.join("posts");
        let post_photos_dir = posts_dir.join("photos");
        std::fs::create_dir_all(&post_photos_dir)?;

        // Create HTM file with post content.
        let htm_path = posts_dir.join(p.get_file_name());
        let html = html::render_post(p, child);
        std::fs::write(htm_path, html)?;

        // Download photos and create hardlinks.
        // TODO: uncomment
        // for ph in &p.photos {
        //     let photo_file_name = ph.get_file_name();

        //     let photo_path = post_photos_dir.join(&photo_file_name);
        //     if !photo_path.exists() {
        //         let mut writer = std::fs::File::create(&photo_path)?;
        //         http::download_image(&ph.url, &mut writer)?;
        //     }

        //     if ph.is_tagged(&child.id) {
        //         let tagged_photo_path = tagged_photos_dir.join(&photo_file_name);
        //         if !tagged_photo_path.exists() {
        //             std::fs::hard_link(&photo_path, tagged_photo_path)?;
        //         }
        //     }
        // }

        i += 1;
        if i % 5 == 0 {
            println!("{} of {} posts stored...", i, total);
        }
    }

    println!("All posts stored");
    Ok(())
}

fn download_tagged_photos(photos: &Vec<Photo>, child: &ChildInfo) -> Result<()> {
    let dir_path = format!("{}/tagged_photos", &child.get_first_name());
    let tagged_photos_dir = std::path::Path::new(dir_path.as_str());
    std::fs::create_dir_all(&tagged_photos_dir)?;

    let total = photos.len();
    let mut i = 0;
    for p in photos {
        let photo_path = tagged_photos_dir.join(p.get_file_name());
        if !photo_path.exists() {
            let mut writer = std::fs::File::create(&photo_path)?;
            http::download_image(&p.url, &mut writer)?;
        }

        i += 1;
        if i % 10 == 0 {
            println!("{} of {} tagged photos downloaded...", i, total);
        }
    }

    println!("All tagged photos downloaded");
    Ok(())
}

fn main() -> Result<()> {
    let env = Config::new();

    let client = http::create_web_client(env.access_token)?;
    
    let child_infos_json = http::fetch_child_infos(&client)?;
    let child_infos = child_info::from_json(child_infos_json)?;

    if child_infos.is_empty() {
        return Err(Error::from("No children found"));
    }
    if child_infos.len() > 1 {
        println!("\nFound children:");
        for (pos, ci) in child_infos.iter().enumerate() {
            println!("{}. {} ({})", pos + 1, ci.full_name_with_institution, ci.institution);
        }
    }
    println!();

    let child = choose_target_child(&child_infos);

    // Before hammering the API, make sure the download folder can be created in principle.
    create_dir(child.get_first_name().as_str())
        .map_err(|e| format!("Cannot create the target folder: {0}", e))?;

    // Fetch posts.
    println!("Fetching posts...");
    let posts = http::fetch_till_exhausted(|older_than| {
        let json = http::fetch_feed(&client, &older_than)?;
        Post::from_feed_json(json, &child.id)
            .map_err(|e| http::Error::from(format!("Failed to deserialize posts: {}", e)))
    })?;
    println!("{0} matching posts found", posts.len());

    // Store posts to disk and downloads related photos.
    if posts.len() > 0 {
        store_posts(&posts, &child)?;
    }

    // Fetch tagged photos info.
    println!("Fetching tagged photos...");
    let tagged_photos = http::fetch_till_exhausted(|older_than| {
        let json = http::fetch_tagged_photos(&client, &&child.id, &older_than)?;
        Photo::from_json_array(json)
            .map_err(|e| http::Error::from(format!("Failed to deserialize tagged photos: {}", e)))
    })?;
    println!("{0} tagged photos found", tagged_photos.len());

    // Download tagged photos.
    if tagged_photos.len() > 0 {
        // TODO: uncomment
        // download_tagged_photos(&client, &tagged_photos, &child)?;
    }

    // Create index.htm
    if posts.len() > 0 || tagged_photos.len() > 0 {
        let name = &child.get_first_name();
        let root_dir = std::path::Path::new(name);

        let htm_path = root_dir.join("index.htm");
        let html = html::render_index(&posts, tagged_photos.len() > 0);
        std::fs::write(htm_path, html)?;
    }

    Ok(())
}
