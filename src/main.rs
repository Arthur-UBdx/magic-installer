use magic_installer::*;

const MINECRAFT_FOLDER: &str = "%appdata%\\.minecraft\\";

fn main() {
    // let config_str = include_str!("..\\src\\config.txt");
    // let config = Config::from_str(config_str);

    // let minecraft_folder = get_env_path(MINECRAFT_FOLDER);
    // let magic_installer_folder = minecraft_folder + "magic_installer\\";

    // create_folder(&magic_installer_folder);
    // let files_path = format!("{}{}", magic_installer_folder, "files.zip");
    // download_file(&files_path, &config.modpack_url).expect("Couldn't download files.zip");

    let mut display = Display::open().unwrap();
    display.main_menu().unwrap();
    display.close().unwrap();
}

//TODO 
// - Add a way to change config.
// - Error Message when not connected to internet.
