use magic_installer::*;

fn main() -> crossterm::Result<()> {
    let config_str = include_str!("..\\src\\config.txt");
    let config = Config::from(config_str);

    create_folder(&config.magic_installer_folder);

    let mut display = Display::open(config)?;
    loop {
        if let AppStatus::Exit = display.main_menu()? {break;}
    }
    display.close()?;
    Ok(())
}

//TODO 
// - Add a way to change config.
// - Error Message when not connected to internet.
