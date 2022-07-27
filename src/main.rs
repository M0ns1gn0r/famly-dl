mod config;
mod console;
mod child_info;
mod post;
mod file_system;
mod http;
mod json;

use child_info::ChildInfo;
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
}

fn choose_target_child(child_infos: Vec<ChildInfo>) -> (String, String) {
    let children_count = child_infos.len();

    if children_count < 2 {
        let child = &child_infos[0];
        return (child.id.clone(), child.get_first_name());
    }

    loop {
        if let Some(child_number) =
            console::choose_number("Select the child (CTRL+C to exit): ", children_count) {
            let child = &child_infos[child_number - 1];
            let child_id = child.id.clone();
            let child_first_name = child.get_first_name();

            println!("{0} is selected ({1})", child_first_name, child_id);
            return (child_id, child_first_name);
        }

        println!("Invalid number")
    }
}

fn get_posts(client: &reqwest::blocking::Client, child_id: &String) -> Result<Vec<Post>> {
    let mut posts = vec![];
    {
        let mut i = 0_u8;
        // TODO: test end condition: very big date.
        let mut older_than = None;
        loop {
            if i > 1 {
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

    let (child_id, child_first_name) = choose_target_child(child_infos);

    create_dir(child_first_name.as_str())
        .expect("Cannot create target folder");

    let posts = get_posts(&client, &child_id)?;
    println!("{0} posts loaded", posts.len());
    for f in posts {
        println!("* {}...", f.text.chars().take(30).collect::<String>());
    }

    Ok(())
}
