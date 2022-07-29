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
use post::{Post, Photo};
use file_system::create_dir;

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

fn store_posts(client: &reqwest::blocking::Client, posts: &Vec<Post>, child: &ChildInfo) -> Result<()> {
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
        let htm_path = posts_dir.join(
            format!("{}.{:02} {}.htm", p.date.year() - 2000, p.date.month(), p.get_title()));
        let html = html::render_post(p, child);
        std::fs::write(htm_path, html)?;

        // Download photos and create hardlinks.
        for ph in &p.photos {
            let photo_file_name = ph.get_file_name();

            let photo_path = post_photos_dir.join(&photo_file_name);
            if !photo_path.exists() {
                let mut writer = std::fs::File::create(&photo_path)?;
                http::download_image(client, &ph.url, &mut writer)?;
            }

            if ph.is_tagged(&child.id) {
                let tagged_photo_path = tagged_photos_dir.join(&photo_file_name);
                if !tagged_photo_path.exists() {
                    std::fs::hard_link(&photo_path, tagged_photo_path)?;
                }
            }
        }

        i += 1;
        if i % 5 == 0 {
            println!("{} of {} posts downloaded...", i, total);
        }
    }

    println!("All posts downloaded");
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
    let posts = http::fetch_till_exhausted(|older_than| {
        let json = http::fetch_feed(&client, &older_than)?;
        Post::from_feed_json(json, &child.id)
            .map_err(|e| http::Error::from(format!("Failed to deserialize posts: {}", e)))
    })?;
    println!("{0} matching posts found", posts.len());

    // Store posts and photos to disk.
    if posts.len() > 0 {
        store_posts(&client, &posts, &child)?;
    }

    // Fetch tagged photos info.
    let tagged_photos = http::fetch_till_exhausted(|older_than| {
        let json = http::fetch_tagged_photos(&client, &&child.id, &older_than)?;
        Photo::from_json_array(json)
            .map_err(|e| http::Error::from(format!("Failed to deserialize tagged photos: {}", e)))
    })?;
    println!("{0} tagged photos found", tagged_photos.len());

    // TODO: store tagged photos.

    Ok(())
}
