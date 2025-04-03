use std::collections::HashMap;
use std::fs::File;

use crate::modules;
use crate::modules::files::create_folder_if_not_exists;

use std::io::Write;

pub const VERSION: &str = "v2.1.2";
pub const MAIN_TITLE: &str = include_str!("../../title.txt");
pub const AUTHOR: &str = "RICHELET Arthur - 2023";
pub const CONTROLS: &str = "↑ ↓ pour naviguer, Entrée pour valider, Esc pour quitter";
pub const BOTTOM_TEXT: &str = "Un installateur pour les gouverner tous";

pub const MINECRAFT_FOLDER_WINDOWS: &str = "%appdata%\\.minecraft\\";
pub const MINECRAFT_FOLDER_LINUX: &str = "%home%/.minecraft/";

pub const DEFAULT_CONFIG: &str = include_str!("../../config.txt");

pub const MAIN_MENU_OPTIONS: &[&str] = &[
    "Installer le modpack",
    "Installer le modloader",
    "Supprimer les fichiers du modpack",
    "Quitter (esc)",
];

pub const FILES_TO_REMOVE: &[&str] = &[
    "mods",
    "config",
    "defaultconfig",
    "kubejs",
    "scripts",
    "panoramas",
];

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- ## SP-PT-001                                                  //
//%--                                                               //
//%-- L'utilitaire doit être capable de fonctionner sur un système  //
//%-- d'exploitation Windows                                        //
//%-- et Linux.                                                     //
//%--                                                               //
//%-----------------------------------------------------------------//
pub const OS_TYPE: &OSType = if cfg!(target_os = "windows") {
    &OSType::Windows
} else if cfg!(target_os = "linux") {
    &OSType::Linux
} else {
    panic!("OS not supported");
};
pub const MINECRAFT_FOLDER: &str = match OS_TYPE {
    OSType::Windows => {
        modules::files::expand_variables(String::from(modules::config::MINECRAFT_FOLDER_WINDOWS))
            .as_str()
    }
    OSType::Linux => {
        modules::files::expand_variables(String::from(modules::config::MINECRAFT_FOLDER_LINUX))
            .as_str()
    }
};

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- ## SP-PT-001                                                  //
//%--                                                               //
//%-- L'utilitaire doit être capable de fonctionner sur un système  //
//%-- d'exploitation Windows                                        //
//%-- et Linux.                                                     //
//%--                                                               //
//%-----------------------------------------------------------------//
#[allow(clippy::upper_case_acronyms)]
pub enum OSType {
    Windows,
    Linux,
}

// ---- Config ---- //
#[derive(Debug)]
pub struct Config {
    pub modpack_url: String,
    pub modloader_url: String,
    pub modloader_execname: String,
    pub minecraft_folder: String,
    pub magic_installer_folder: String,
    pub debug: bool,
}

impl Config {
    /// Create a new Config instance from a string containing key-value pairs, separated by = and \n.
    pub fn from(config: &str) -> Config {
        let config = Config::parse_hashmap(config, "\n", "=");

        let magic_installer_folderpath =
            format!("{}{}", get_env_path(MINECRAFT_FOLDER), "magic_installer/");
        create_folder_if_not_exists(magic_installer_folderpath.as_str());

        Config {
            modpack_url: config.get("modpack_url").unwrap().to_string(),
            modloader_url: config.get("modloader_url").unwrap().to_string(),
            modloader_execname: config.get("modloader_execname").unwrap().to_string(),
            minecraft_folder: MINECRAFT_FOLDER,
            magic_installer_folder: magic_installer_folderpath,
            debug: false,
        }
    }

    pub fn enable_debug(mut self, enable: bool) {
        self.debug = enable;
    }

    /// Parse a string containing key-value pairs, with custom separators.
    fn parse_hashmap(
        target: &str,
        entries_separator: &str,
        key_value_separator: &str,
    ) -> HashMap<String, String> {
        let mut result: HashMap<String, String> = HashMap::new();
        let entries = target.split(entries_separator);
        entries.for_each(|e| {
            if let Some((k, v)) = e.split_once(key_value_separator) {
                result.insert(k.trim().to_string(), v.trim().to_string());
            }
        });
        result
    }

    /// Log a message to the debug file.
    pub fn log(&mut self, message: &str) {
        modules::files::append_to_file(
            format!("{}{}", MINECRAFT_FOLDER, "magic_installer/debug.txt").as_str(),
            message,
        )
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::from(DEFAULT_CONFIG)
    }
}
