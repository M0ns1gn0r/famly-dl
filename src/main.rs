mod config;
mod console;
mod child_info;
mod feed_item;
mod file_system;
mod http;
mod json;

use child_info::ChildInfo;
use config::Config;
use error_chain::error_chain;
use feed_item::FeedItem;
use file_system::create_dir;

error_chain! {
    links {
        ChildInfo(child_info::Error, child_info::ErrorKind);
        FeedItem(feed_item::Error, feed_item::ErrorKind);
        Http(http::Error, http::ErrorKind);
    }
}

fn choose_target_child(child_infos: Vec<ChildInfo>) -> (String, String) {
    let child_id: String;
    let child_first_name: String;
    loop {
        if let Some(child_number) = console::choose_number(
            "Enter the target child number (CTRL+C to exit): ",
            child_infos.len()) {
            let child = &child_infos[child_number - 1];
            child_id = child.id.clone();
            child_first_name = child.get_first_name();
            break;
        } else {
            println!("Invalid number")
        }
    }

    (child_id, child_first_name)
}

fn fetch_feed_items(client: &reqwest::blocking::Client) -> Result<Vec<FeedItem>> {
    let mut feed_items = vec![];
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

            let feed_json = http::fetch_feed(&client, &older_than)?;
            let (feed_item_portion, last_item_date) = feed_item::from_json(feed_json)?;

            feed_items.extend(feed_item_portion);

            if last_item_date.is_none() {
                // The feed has ended.
                break;
            } else {
                older_than = last_item_date;
            }
        }
    }

    Ok(feed_items)
}

fn main() -> Result<()> {
    let env = Config::new();

    console::confirm("Press ENTER to start...");

    let client = http::create_web_client(env.access_token)?;
    
    let child_infos_json = http::fetch_child_infos(&client)?;
    let child_infos = child_info::from_json(child_infos_json)?;

    println!("\nFound children:");
    for (pos, ci) in child_infos.iter().enumerate() {
        println!("{}. {} ({})", pos + 1, ci.full_name_with_institution, ci.institution);
    }
    println!();

    let (child_id, child_first_name) = choose_target_child(child_infos);
    println!("{0} is selected ({1})", child_first_name, child_id);

    create_dir(child_first_name.as_str())
        .expect("Cannot create target folder");

    let feed_items = fetch_feed_items(&client)?;
    println!("{0} feed items loaded", feed_items.len());
    for f in feed_items {
        println!("* {}...", f.text.chars().take(30).collect::<String>());
    }

    Ok(())
}
