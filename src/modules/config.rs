#[allow(unused_imports)]
use crate::modules::{app, files, utils};

use crate::modules;
use crate::modules::utils::LogExcept;

#[cfg(target_os = "windows")]
pub const MINECRAFT_FOLDER_WINDOWS: &str = "%appdata%\\.minecraft\\";
#[cfg(target_os = "linux")]
pub const MINECRAFT_FOLDER_LINUX: &str = "$HOME/.minecraft/";

pub const DEFAULT_CONFIG: &str = include_str!("../../assets/default_config.txt");

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- ## SP-CF-002                                                  //
//%--                                                               //
//%-- Le dossier de jeu minecraft doit pouvoir être spécifié par    //
//%-- l'utilisateur.                                                //
//%--                                                               //
//%-----------------------------------------------------------------//
/// Returns the path to the Minecraft folder.
/// If the environment variable MINECRAFT_DIR is set, it will be used.
/// Otherwise, the default path will be used based on the OS.
pub fn get_minecraft_folder() -> String {
    let path: String;
    // if the environment variable MINECRAFT_DIR is set, use it
    if let Ok(override_path) = std::env::var("MINECRAFT_DIR") {
        path = modules::utils::expand_variables(override_path)
    } else {
        // else use the default path based on the OS
        #[cfg(target_os = "windows")]
        {
            path = modules::utils::expand_variables(String::from(MINECRAFT_FOLDER_WINDOWS));
        }
        #[cfg(target_os = "linux")]
        {
            path = modules::utils::expand_variables(String::from(MINECRAFT_FOLDER_LINUX));
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            panic!("Unsupported OS");
        }
    }
    // if the path does not end with a /, add it
    // this is important for the path to be valid
    if path.ends_with('/') {
        path
    } else {
        format!("{}{}", path, "/")
    }
}

// ---- Config ---- //
#[derive(Debug)]
pub struct Config {
    pub modpack_url: String,
    pub modloader_url: String,
    pub modloader_execname: String,
    pub files_to_overwrite: Vec<String>,
    pub minecraft_folder: String,
    pub magic_installer_folder: String,
}

impl Config {
    /// Create a new Config instance from a string containing key-value pairs, separated by = and \n.
    pub fn from(config: &str) -> Config {
        let config = utils::parse_hashmap(config, "\n", "=");

        utils::log(&format!("LOG: Config:\n{:?}", config)).unwrap();

        Config {
            modpack_url: config
                .get("modpack_url")
                .log_expect("Could not get 'modpack_url'")
                .to_string(),
            modloader_url: config
                .get("modloader_url")
                .log_expect("Could not get 'modloader_url'")
                .to_string(),
            modloader_execname: config
                .get("modloader_execname")
                .log_expect("Could not get 'modloader_execname'")
                .to_string(),
            files_to_overwrite: config
                .get("folders_to_overwrite")
                .log_expect("Could not get 'folders_to_overwrite'")
                .split(",")
                .map(|s| s.trim().to_string())
                .collect(),
            minecraft_folder: get_minecraft_folder(),
            magic_installer_folder: format!("{}{}", get_minecraft_folder(), "magic_installer/"),
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::from(DEFAULT_CONFIG)
    }
}

#[cfg(test)]
mod tests {
    #[allow(unused_imports)]
    use super::*;
}
