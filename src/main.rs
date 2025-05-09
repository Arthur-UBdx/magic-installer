mod modules {
    pub mod app;
    pub mod config;
    pub mod files;
    pub mod utils;
}

#[allow(unused_imports)]
use modules::{utils, app, config, files};
use modules::utils::LogExcept;
use crate::modules::app::{AppStatus, Display};
use crate::modules::config::Config;

fn main() -> crossterm::Result<()> {

    let config_string: String;
    let config: Config;

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
    let magic_installer_folder = format!(
        "{}{}",
        modules::config::get_minecraft_folder(),
        "magic_installer/"
    );
    match modules::files::create_folder_if_not_exists(&magic_installer_folder) {
        modules::files::FileStatus::NoChange => {}
        modules::files::FileStatus::Ok => {}
        modules::files::FileStatus::Error(e) => {
            panic!("Error creating magic_installer folder {} : {}", &magic_installer_folder, e);
        }
    }

    //-----
    // on check si le fichier de config existe
    // sinon on le crée
    //-----
    match modules::files::create_file_if_not_exists(&format!(
        "{}magic_installer/config.txt",
        modules::config::get_minecraft_folder())) {
        //-----
        // si le fichier n'existe pas, on le crée
        // et on met la config par défaut
        //-----
        modules::files::FileStatus::Ok => {
            config_string = String::from(modules::config::DEFAULT_CONFIG);
            files::append_to_file(
                &format!(
                        "{}magic_installer/config.txt",
                        modules::config::get_minecraft_folder()), &config_string
                )
                .unwrap_or_else(|e| {
                    panic!("Error creating config file : {}", e);
                });
        }
        //-----
        // si le fichier existe, on le lit
        //-----
        modules::files::FileStatus::NoChange => {
            config_string = modules::files::read_file(format!(
                "{}{}",
                modules::config::get_minecraft_folder(),
                "magic_installer/config.txt"
            ).as_str()).log_expect("Error reading config file");
        }
        //-----
        // si le fichier n'a pas pu être créé, on affiche une erreur
        // et on quitte le programme
        //-----
        modules::files::FileStatus::Error(e) => {
            panic!("Error creating config file : {}", e);
        }
    }
    // on charge la config lue ou par défaut
    // et on la parse    
    config = Config::from(&utils::remove_comments(config_string));


    let mut display = Display::open(config)?;
    loop {
        if let AppStatus::Exit = display.main_menu()? {
            break;
        }
        crossterm::event::read().unwrap();
    }
    Display::close()?;
    Ok(())
}
