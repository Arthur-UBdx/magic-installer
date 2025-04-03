mod modules {
    pub mod app;
    pub mod config;
    pub mod files;
}

use crate::modules::app::{AppStatus, Display};
use crate::modules::config::Config;
use std::env;

fn main() -> crossterm::Result<()> {

    let mut config_string = String::default();
    
    //%-----------------------------------------------------------------//
    //%--                                                               //
    //%-- SP-FN-005:                                                    //
    //%--                                                               //
    //%-- Le programme doit pouvoir être lancé en mode debug au travers //
    //%-- d'un argument de ligne de commande.                           //
    //%--                                                               //
    //%-- Implémentation:                                               //
    //%--                                                               //
    //%-- Parsing des arguments de la ligne de commande, si l'argument  //
    //%-- -debug est présent, le mode debug est activé.                 //
    //%--                                                               //
    //%-----------------------------------------------------------------//

    let mut debug: bool = false;

    let args: Vec<String> = env::args().collect();

    if args.len() > 1 && args[1].as_str() == "-debug" {
        debug = true;
    }

    //%-----------------------------------------------------------------//
    //%--                                                               //
    //%-- SP-FN-001:                                                    //
    //%--                                                               //
    //%-- L'utilitaire doit lire un fichier de configuration pour       //
    //%-- obtenir les informations                                      //
    //%-- nécessaires à son fonctionnement. Sinon il doit utiliser une  //
    //%-- configuration par défaut.                                     //
    //%--                                                               //
    //%-----------------------------------------------------------------//
    match modules::files::create_folder_if_not_exists(format!(
        "{}{}",
        modules::config::MINECRAFT_FOLDER,
        "magic_installer/"
    ).as_str()) {
        modules::files::FileStatus::FileCreated => {}
        modules::files::FileStatus::Exists => {}
        modules::files::FileStatus::Error => {
            panic!("Error creating magic_installer folder");
        }
    }

    //-----
    // on check si le fichier de config existe
    // sinon on le crée
    //-----
    match modules::files::create_file_if_not_exists(format!(
        "{}{}",
        modules::config::MINECRAFT_FOLDER,
        "magic_installer/config.txt"
    ).as_str()) {
        //-----
        // si le fichier n'existe pas, on le crée
        // et on met la config par défaut
        //-----
        modules::files::FileStatus::FileCreated => {
            config_string = String::from(modules::config::DEFAULT_CONFIG);
        }
        //-----
        // si le fichier existe, on le lit
        //-----
        modules::files::FileStatus::Exists => {
            config_string = modules::files::read_file(format!(
                "{}{}",
                modules::config::MINECRAFT_FOLDER,
                "magic_installer/config.txt"
            ).as_str()).expect("Error reading config file");
        }
        //-----
        // si le fichier n'a pas pu être créé, on affiche une erreur
        // et on quitte le programme
        //-----
        modules::files::FileStatus::Error => {
            panic!("Error creating config file");
        }
    }

    // on charge la config lue ou par défaut
    // et on la parse    
    let config: Config = Config::from(&config_string);
    
    // on active le debug si demandé
    config.enable_debug(debug);

    let mut display = Display::open(&config)?;
    loop {
        if let AppStatus::Exit = display.main_menu()? {
            break;
        }
        crossterm::event::read().unwrap();
    }
    display.close()?;
    Ok(())
}

//TODO
// - Add a way to change config.
