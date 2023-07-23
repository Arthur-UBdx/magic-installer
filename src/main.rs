mod config;
mod ui;
mod files;

use crate::config::Config;
use crate::ui::{Display, AppStatus};
use crate::files::create_folder;
use std::fs::File;
use std::env;

fn main() -> crossterm::Result<()> {
    let config_str = include_str!("../config.txt");
    let config: Config;
    
    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        match args[1].as_str() {
            "debug" => {
                let this_exe = std::env::current_exe().unwrap();
                let this_folder = this_exe.parent().unwrap();
                let mut path = this_folder.to_path_buf();
                path.push("log.txt");
                let mut logfile = File::create(path)?;
                config = Config::debug_from(config_str, &mut logfile);
            }
            _ => {
                config = Config::from(config_str);
            }
        }
    } else {
        config = Config::from(config_str);
    }
    
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
// - Error Message when not connected to internet.
