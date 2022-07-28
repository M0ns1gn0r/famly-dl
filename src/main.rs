mod config;
mod console;
mod child_info;
mod post;
mod file_system;
mod http;
mod json;

use child_info::ChildInfo;
use chrono::Datelike;
use config::Config;
use error_chain::error_chain;
use post::Post;
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

fn get_posts(client: &reqwest::blocking::Client, child_id: &String) -> Result<Vec<Post>> {
    let mut posts = vec![];
    {
        let mut i = 0_u8;
        let mut older_than = None;
        loop {
            if i > 2 {
                // TODO: remove this artificial break condition.
                break;
            }
            i += 1;

            let feed_json = http::fetch_feed(client, &older_than)?;
            let (posts_portion, last_item_date) = post::from_feed_json(feed_json, child_id)?;

            posts.extend(posts_portion);

            if last_item_date.is_none() {
                // The feed has ended.
                break;
            } else {
                older_than = last_item_date;
            }
        }
    }

    Ok(posts)
}

fn download_posts(posts: &Vec<Post>, child: &ChildInfo) -> Result<()> {
    let name = &child.get_first_name();
    let root_dir = std::path::Path::new(name);

    let total = posts.len();
    let mut i = 0;
    for p in posts {
        let dir = root_dir.join("posts");
        std::fs::create_dir_all(&dir)?;

        let mut photos = String::new();
        for ph in &p.photos {
            let img = format!(r#"<a target="_blank" href="{0}">
    <img src="{0}" class="img-thumbnail" style="max-height: 240px" />
</a>"#, ph.url);
            photos.push_str(img.as_str());
        }

        let mut comments = String::new();
        if !p.comments.is_empty() {
            comments.push_str(r#"<hr /><h4 class="mb-3">Comments:</h4>"#);
            for c in &p.comments {
                let comment = format!(r#"<div class="bg-light border p-2 mb-1 rounded-3">
    💬<b class="ms-1">{0}</b>
    <br>
    <div style="white-space: pre-line;">{1}</div>
</div>
"#, &c.author,&c.text);
                comments.push_str(comment.as_str());
            }
        }

        let html = format!(r#"
<!doctype html>
<html>
<head>
<meta charset="utf-8">
<link href="https://cdn.jsdelivr.net/npm/bootstrap@5.2.0/dist/css/bootstrap.min.css" rel="stylesheet" integrity="sha384-gH2yIJqKdNHPEq0n4Mqa/HGKIhSkIHeL5AyhkYV8i59U5AR6csBvApHHNl/vI1Bx" crossorigin="anonymous">
</head>
<body>
<div class="container py-3" style="max-width: 1000px;">
    <p>
        <b>{author}</b>
        <br>
        {date}
    </p>
    <hr />
    <div style="white-space: pre-line;">{text}</div>
    <br />
    <div>{photos}</div>
    {comments}
</div>
</body>
</html>"#,
            author = p.author,
            date = p.date.with_timezone(&chrono::Local).to_rfc2822(),
            text = p.text,
            comments = comments);

        let file = dir.join(format!("{}.{:02} {}.htm", p.date.year() - 2000, p.date.month(), p.get_title()));
        std::fs::write(file, html)?;

        i += 1;
        if i % 10 == 0 {
            println!("{} of {} posts downloaded...", i, total);
        }
    }

    println!("All posts downloaded");
    Ok(())
}

fn main() -> Result<()> {
    let env = Config::new();

    console::confirm("Press ENTER to start...");

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

    let posts = get_posts(&client, &child.id)?;
    println!("{0} matching posts found", posts.len());

    if posts.len() > 0 {
        download_posts(&posts, &child)?;
    }

    Ok(())
}
