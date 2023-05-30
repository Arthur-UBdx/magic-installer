use magic_installer::*;

const MINECRAFT_FOLDER: &str = "%appdata%\\.minecraft\\";

fn main() -> crossterm::Result<()> {
    let config_str = include_str!("..\\src\\config.txt");
    let config = Config::from(config_str);

    let minecraft_folder = get_env_path(MINECRAFT_FOLDER);
    let magic_installer_folder = minecraft_folder + "magic_installer\\";

    create_folder(&magic_installer_folder);
    let files_path = format!("{}{}", magic_installer_folder, "files.zip");

    let display = Display::open()?;
    display.download_page(&files_path, &config.modpack_url)?;
    std::thread::sleep(std::time::Duration::from_secs(5));
    // display.main_menu()?;
    display.close()?;
    Ok(())
}

//TODO 
// - Add a way to change config.
// - Error Message when not connected to internet.
