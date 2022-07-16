use std::env;

pub struct Config {
    pub access_token: String,
    pub default_target_folder: Option<String>,
}

impl Config {
    pub fn new() -> Config {        
        let access_token = env::var("FAMLY_ACCESS_TOKEN").unwrap();
        let default_target_folder = env::var("FAMLY_TARGET_FOLDER").ok();

        Config { access_token, default_target_folder }
    }
}