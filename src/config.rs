use std::collections::HashMap;
use std::fs::File;
use std::io::Write;

pub const VERSION: &str = "v2.0.1";
pub const MAIN_TITLE: &str = include_str!("../title.txt");
pub const AUTHOR: &str = "RICHELET Arthur - 2023";
pub const BOTTOM_TEXT: &str = "Un installateur pour les gouverner tous";

pub const MINECRAFT_FOLDER: &str = "%appdata%\\.minecraft\\";
pub const MAIN_MENU_OPTIONS: &[&str] = &["Installer le modpack", "Installer fabric", "Supprimer les fichiers du modpack", "Quitter (esc)"];
pub const FILES_TO_REMOVE: &[&str] = &["mods", "config"];

// ---- Config ---- //

#[derive(Debug)]
pub struct Config {
    pub modpack_url: String,
    pub modloader_url: String,
    pub minecraft_folder: String,
    pub magic_installer_folder: String,
}

impl Config {
    pub fn from(config: &str) -> Config {
        let config = Config::parse_hashmap(config, "\n", "=");

        Config {
            modpack_url: config.get("modpack_url").unwrap().to_string(),
            modloader_url: config.get("modloader_url").unwrap().to_string(),
            minecraft_folder: get_env_path(MINECRAFT_FOLDER),
            magic_installer_folder: format!("{}{}", get_env_path(MINECRAFT_FOLDER), "magic_installer\\"),
        }
    }

    pub fn debug_from(config: &str, log_file: &mut File) -> Config {
        let config = Config::parse_hashmap(config, "\n", "=");

        writeln!(log_file, "modpack_url: {}", config.get("modpack_url").unwrap()).unwrap();
        writeln!(log_file, "modloader_url: {}", config.get("modloader_url").unwrap()).unwrap();

        Config {
            modpack_url: config.get("modpack_url").unwrap().to_string(),
            modloader_url: config.get("modloader_url").unwrap().to_string(),
            minecraft_folder: get_env_path(MINECRAFT_FOLDER),
            magic_installer_folder: format!("{}{}", get_env_path(MINECRAFT_FOLDER), "magic_installer\\"),
        }
    }

    fn parse_hashmap(target: &str, entries_separator: &str, key_value_separator: &str) -> HashMap<String, String> {
        let mut result: HashMap<String, String> = HashMap::new();
        let entries = target.split(entries_separator);
        entries.for_each(|e| {
            if let Some((k,v)) = e.split_once(key_value_separator) {
                result.insert(
                    k.trim().to_string(),
                    v.trim().to_string()
                );
            }
        });
        result
    }
}

pub fn get_env_path(path: &str) -> String {
    if path.starts_with('%') {
        let path_splitted: Vec<&str> = path.split('%').collect();
        let var: &str = &path_splitted[1].to_uppercase();
        let path = match std::env::var(var) {
            Ok(path) => path,
            Err(_) => panic!("Environnement variable '{}' not found", var),
        };
        return path + path_splitted[2];
    }
    path.to_string()
}