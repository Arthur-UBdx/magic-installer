mod config;
mod app;
mod files;

use crate::config::Config;
use crate::app::{Display, AppStatus};
use crate::files::create_folder;
use std::env;

fn main() -> crossterm::Result<()> {
    let config_str = include_str!("../config.txt");
    let mut debug: bool = false;
    
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "debug" => {debug = true;}
            _ => {}
        }
    }

    let config: Config = Config::from(config_str, debug);
    create_folder(&config.magic_installer_folder);

    let mut display = Display::open(config)?;
    loop {
        if let AppStatus::Exit = display.main_menu()? {break;}
        crossterm::event::read().unwrap();
    }
    display.close()?;
    Ok(())
}

//TODO 
// - Add a way to change config.
