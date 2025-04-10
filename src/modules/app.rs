#[allow(unused_imports)]
use crate::modules::{app, config, files, utils};

use files::{
    download_file, launch_executable, remove_folder, unzip_file, DownloadStatus, FileStatus,
};

use utils::LogExcept;
use std::panic;
use std::io::{self, Write};
use std::sync::Arc;
use std::thread;
use std::thread::sleep;
use std::time::Duration;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{Attribute, Color, Print, PrintStyledContent, StyledContent, Stylize},
    terminal,
};

use std::sync::mpsc;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = "RICHELET Arthur - 2025";
const CONTROLS: &str = "↑ ↓ pour naviguer, Entrée pour valider, Esc pour quitter";
const BOTTOM_TEXT: &str = "Un installateur pour les gouverner tous";
const MAIN_MENU_OPTIONS: &[&str] = &[
    "Installer le modpack",
    "Installer le modloader",
    "Supprimer les fichiers du modpack",
    "Ouvrir le fichier de configuration",
    "Quitter (esc)",
    ];

#[cfg(target_os = "windows")]
const MAIN_TITLE: &str = include_str!("../../assets/title.txt");
#[cfg(target_os = "linux")]
const MAIN_TITLE: &str = include_str!("../../assets/title.txt");


pub enum AppStatus {
    Loop,
    Exit,
}

pub struct Display<'a> {
    terminal_width: u16,
    terminal_height: u16,
    config: &'a config::Config,
}

impl<'a> Display<'a> {
    /// Creates a new Display instance and enters the alternate screen mode.
    /// This function should be called at the beginning of the application.
    /// It initializes the terminal size and sets up the display.
    pub fn open(config: &'a config::Config) -> crossterm::Result<Display<'a>> {
        Display::setup_panic_hook();
        
        #[cfg(target_os = "linux")] {
            terminal::enable_raw_mode()?; // Enable raw mode to capture input on linux
            execute!(
                io::stdout(),
                event::EnableMouseCapture,
            )?;
        }        
        execute!(
            io::stdout(), 
            terminal::EnterAlternateScreen,
            cursor::Hide, 
        )?;
        Ok(Display {
            terminal_width: terminal::size()?.0,
            terminal_height: terminal::size()?.1,
            config,
        })
    }
    
    /// Closes the display and exits the alternate screen mode.
    /// This function should be called when the application is done using the terminal.
    /// It restores the terminal to its original state.
    pub fn close() -> crossterm::Result<()> {
        #[cfg(target_os = "linux")] {
            terminal::disable_raw_mode()?; // Disable raw mode
            execute!(
                io::stdout(),
                event::DisableMouseCapture,    
            )?;
        }
        execute!(
            io::stdout(),
            terminal::LeaveAlternateScreen, 
            cursor::Show                     
        )?;
        Ok(())
    }
    
    fn setup_panic_hook() {
        panic::set_hook(Box::new(move |info| {
            #[allow(unused_must_use)] {
                Display::close();
                let message = format!("Unrecoverable error: {}", info);
                utils::log(&message);
                println!("{}", message);
            }
        }))
    }

    /// Writes a string to the terminal, centered based on the current terminal width.
    fn write_centered(&self, text: &str) -> crossterm::Result<()> {
        let padding: usize = (self.terminal_width.saturating_sub(text.len() as u16) / 2) as usize;
        execute!(io::stdout(), Print(" ".repeat(padding)), Print(text))?;
        Ok(())
    }

    /// Writes a styled string to the terminal, centered based on the current terminal width.
    /// The string is styled using the `StyledContent` type from crossterm.
    /// This function is useful for displaying text with colors and attributes.
    fn write_stylized_centered(&self, stylized_text: StyledContent<&str>) -> crossterm::Result<()> {
        let padding: usize = (self
            .terminal_width
            .saturating_sub(stylized_text.content().len() as u16)
            / 2) as usize;
        execute!(
            io::stdout(),
            Print(" ".repeat(padding)),
            PrintStyledContent(stylized_text)
        )?;
        Ok(())
    }

    // MAIN MENU
    /// Displays the main menu of the application.
    /// This function handles user input and updates the display accordingly.
    /// It uses a loop to wait for user input and redraws the menu based on the selected option.
    /// It returns an `AppStatus` indicating whether to continue or exit the application.
    /// The main menu consists of several options, including installing a modpack, installing a modloader,
    /// removing files, and exiting the application.
    /// The function also handles keyboard events for navigation and selection.
    pub fn main_menu(&mut self) -> crossterm::Result<AppStatus> {
        let options = MAIN_MENU_OPTIONS;
        let options_len = options.len();

        let mut selected = 0;
        let key_pressed: KeyCode;

        // Main drawing
        self.draw_main_menu(selected, options)?;

        // Event loop
        loop {
            if event::poll(Duration::from_millis(3000))? {
                match event::read().unwrap() {
                    Event::Key(KeyEvent { code, .. }) => {
                        match code {
                            KeyCode::Up => {
                                selected = (selected + options_len - 1) % options_len;
                                self.draw_main_options(selected, options)?;
                            }
                            KeyCode::Down => {
                                selected = (selected + 1) % options_len;
                                self.draw_main_options(selected, options)?;
                            }
                            KeyCode::Enter => {
                                key_pressed = KeyCode::Enter;
                                break;
                            }
                            KeyCode::Esc => {
                                key_pressed = KeyCode::Esc;
                                break;
                            }
                            _ => {}
                        }
                    }
                    Event::Resize(width, height) => {
                        self.terminal_width = width;
                        self.terminal_height = height;
                        self.draw_main_menu(selected, options)?;
                    }
                    _ => {}
                }
            }
            execute!(io::stdout(), cursor::Hide)?;
        }

        // Handle key pressed
        // key_pressed = event::read().unwrap();
        // key_pressed = KeyCode::Enter;
        // key_pressed = KeyCode::Esc;
        // key_pressed = KeyCode::Up;
        // key_pressed = KeyCode::Down;
        match key_pressed {
            KeyCode::Esc => return Ok(AppStatus::Exit),
            _ => {
                match selected {
                    0 => {
                        // install the modpack
                        let filename: &str = "modpack.zip";
                        let filepath: String =
                            format!("{}{}", &self.config.minecraft_folder, filename);
                        let folders: &[&str] = &self.config.files_to_overwrite
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<&str>>();

                        utils::log(&format!("modpack zip file path: {}", &filepath)).unwrap();
                        utils::log(&format!("files to remove path: {:?}", &folders)).unwrap();

                        self.remove_files_page(&self.config.minecraft_folder, folders)?;
                        self.download_page(&filepath, &self.config.modpack_url)
                            .log_expect("Error when launching Download page");
                        self.unzip_page(filename, &self.config.minecraft_folder)
                            .log_expect("Error when launching Unzip page");
                    }
                    1 => {
                        // install the modloader (fabric/forge)
                        let filename: &str = "modloader.zip";
                        let filepath: String =
                            format!("{}{}", &self.config.magic_installer_folder, filename);
                        let executable_path: String = format!(
                            "{}{}",
                            &self.config.magic_installer_folder, self.config.modloader_execname
                        );

                        utils::log(&format!("modloader zip path: {}", &filepath)).unwrap();
                        utils::log(&format!("modloader exec path: {}", &executable_path)).unwrap();
                        utils::log(&format!("magic_installer folder path: {}",&self.config.magic_installer_folder)).unwrap();

                        self.download_page(&filepath, &self.config.modloader_url)
                            .log_expect("Error when launching Download page");
                        self.unzip_page(filename, &self.config.magic_installer_folder)
                            .log_expect("Error when launching Unzip page");
                        self.executable_page(&executable_path)
                            .log_expect("Error when launching Executable page");
                    }
                    2 => {
                        // remove all files
                        let folders = &self.config.files_to_overwrite
                            .iter()
                            .map(|s| s.as_str())
                            .collect::<Vec<&str>>();
                        self.remove_files_page(&self.config.minecraft_folder, folders)?;
                    } // open config file
                    3 => {
                        let config_path: String = format!("{}magic_installer/config.txt", self.config.minecraft_folder);
                        utils::log(&format!("Opening config file path: {}", &config_path)).unwrap();

                        // Open the config file in the default text editor depending on the OS
                        #[cfg(target_os = "windows")] {
                            std::process::Command::new("notepad")
                                .arg(&config_path)
                                .spawn()
                                .log_expect(&format!("Failed to open config file {}", &config_path));
                        }
                        #[cfg(target_os = "linux")] {
                            // deactivate the raw mode to open the file
                            terminal::disable_raw_mode()?;
                            execute!(
                                io::stdout(),
                                event::DisableMouseCapture,
                                cursor::Show                     
                            )?;
                            // run vim as a child process and wait for it to be closed
                            // this is a blocking call
                            std::process::Command::new("vim")
                                .arg(&config_path)
                                .spawn()
                                .log_expect(&format!("Failed to open config file {}", &config_path))
                                .wait()
                                .log_expect("Failed to wait for vim to finish");

                            // reenable the raw mode for the rest of the program
                            terminal::enable_raw_mode()?;
                            execute!(io::stdout(), 
                                event::EnableMouseCapture,
                                terminal::Clear(terminal::ClearType::All),
                                cursor::Hide
                            )?;

                            execute!(io::stdout(), cursor::MoveTo(0, self.terminal_height / 2 - 1))?;
                            self.write_centered("Appuyez sur n'importe quelle touche pour continuer")?;
                        }
                        #[cfg(not(any(target_os = "windows", target_os = "linux")))] {
                            panic!("Unsupported OS");
                        }
                    } // exit
                    4 => {
                        // clear the screen
                        execute!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
                        execute!(io::stdout(), cursor::MoveTo(0, 0))?;
                        // exit the program
                        return Ok(AppStatus::Exit);
                    },
                    _ => {}
                }
            }
        };
        Ok(AppStatus::Loop)
    }

    /// Draws the main menu of the application.
    /// This function takes the selected index and the options to display.
    fn draw_main_menu(&self, selected: usize, options: &[&str]) -> crossterm::Result<()> {
        let title: &str = MAIN_TITLE;
        let author: String = format!("{} - {}", AUTHOR, VERSION);
        let bottom_text: &str = BOTTOM_TEXT;
        let controls: &str = CONTROLS;

        let first_line = 100; //title.lines().next().unwrap(); // 100 is the length of the first line of the title
        let padding = (self.terminal_width.saturating_sub(first_line) / 2) as usize;
        let mut stdout = io::stdout();
        
        // disable the raw mode to print the title for unix otherwise it will be messed up because of unicode characters
        #[cfg(target_os = "linux")] {
            terminal::disable_raw_mode()?;
        }
        
        execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::Hide)?;
        execute!(stdout, cursor::MoveTo(0, 0))?;
        title.lines().for_each(|line| {
                queue!(
                    stdout,
                    Print(" ".repeat(padding)),
                    PrintStyledContent(line.with(Color::Blue)),
                    Print("\n")
                )
                .unwrap();
        });
        stdout.flush()?;
        
        // reenable the raw mode for the rest of the program
        #[cfg(target_os = "linux")] {
            terminal::enable_raw_mode()?;
        }

        execute!(stdout, cursor::MoveTo(0, 15))?;
        self.write_stylized_centered(author.as_str().with(Color::Blue).attribute(Attribute::Dim))?;
        execute!(stdout, cursor::MoveTo(0, 17))?;
        self.write_stylized_centered(controls.with(Color::DarkGrey).attribute(Attribute::Dim))?;
        execute!(stdout, cursor::MoveTo(0, self.terminal_height))?;
        self.write_stylized_centered(bottom_text.with(Color::DarkGrey).attribute(Attribute::Dim))?;

        self.draw_main_options(selected, options)?;
        Ok(())
    }

    /// Draw the main options of the menu, with the selected option highlighted.
    /// This function takes the selected index and the options to display.
    pub fn draw_main_options(&self, selected: usize, options: &[&str]) -> crossterm::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, cursor::MoveTo(0, 4))?;
        options.iter().enumerate().for_each(|(index, option)| {
            execute!(stdout, cursor::MoveTo(0, 20 + 2 * index as u16)).unwrap();
            execute!(stdout, terminal::Clear(terminal::ClearType::CurrentLine)).unwrap();
            if index == selected {
                self.write_stylized_centered(
                    format!("> {} <", option)
                        .as_str()
                        .with(Color::Green)
                        .attribute(Attribute::Bold),
                )
                .unwrap();
            } else {
                self.write_centered(option).unwrap();
            }
        });
        stdout.flush()?;
        Ok(())
    }

    /// Download page for the modpack or modloader
    /// This function handles the download process and displays a progress bar.
    /// It uses a separate thread to download the file and communicates the progress
    /// using a channel.
    pub fn download_page(&self, path: &str, url: &str) -> crossterm::Result<()> {
        let path: Arc<String> = Arc::new(path.to_owned());
        let url: Arc<String> = Arc::new(url.to_owned());

        let mut stdout: io::Stdout = io::stdout();
        let height: u16 = (self.terminal_height as f32 / 2.0) as u16;
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2)
        )?;

        self.write_centered("Téléchargement en cours...")?; //lang
        execute!(stdout, cursor::MoveTo(0, height))?;
        self.write_centered("Préparation du téléchargement")?; //lang

        // execute!(stdout, cursor::MoveTo(0, height*2u16))?;
        // self.write_stylized_centered("Si le télécharchement semble rester à 0%, Ctrl+C peut débloquer le programme".with(Color::DarkGrey))?; //lang

        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            download_file(&path, &url, tx).expect("Couldn't download file");
        });

        loop {
            match rx.try_recv() {
                Ok(DownloadStatus::Downloading(percentage)) => {
                    execute!(stdout, cursor::MoveTo(0, height))?;
                    self.write_centered(&format!(
                        "{} {}%",
                        Display::download_bar(percentage),
                        (percentage * 100.0) as u32
                    ))?;
                }
                Ok(DownloadStatus::Downloaded) => {
                    break;
                }
                Ok(DownloadStatus::Error(error)) => {
                    execute!(stdout, cursor::MoveTo(0, height))?;
                    self.write_stylized_centered(
                        format!("Erreur: {}", error)
                            .as_str()
                            .with(Color::Red)
                            .attribute(Attribute::Bold),
                    )
                    .unwrap();
                    sleep(Duration::from_secs(2));
                    execute!(stdout, terminal::Clear(terminal::ClearType::All))?;
                    execute!(stdout, cursor::MoveTo(0, height))?;
                    return Err(io::Error::new(io::ErrorKind::Other, "Download Error"));
                }
                Err(_) => {}
            }
        }
        handle.join().unwrap();

        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, height - 2)
        )?;

        self.write_centered("Téléchargement terminé !")?; //lang
        sleep(Duration::from_secs(1));
        Ok(())
    }

    /// Download bar element
    fn download_bar(percentage: f32) -> String {
        let bar_length = 50;
        let mut bar = String::new();
        bar.push('[');
        for i in 0..bar_length {
            if (i as f32 / bar_length as f32) < percentage {
                bar.push('=');
            } else {
                bar.push(' ');
            }
        }
        bar.push(']');
        bar
    }

    /// Loading page for unzipping files
    pub fn unzip_page(&self, filename: &str, folderpath: &str) -> crossterm::Result<()> {
        let height = self.terminal_height / 2u16;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2)
        )?;

        self.write_centered("Installation en cours...")?; //lang
        unzip_file(filename, folderpath)?;

        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2)
        )?;

        self.write_centered("Installation terminée...")?; //lang
        Ok(())
    }

    /// Launch the executable file page
    pub fn executable_page(&mut self, filepath: &str) -> crossterm::Result<()> {
        let height = self.terminal_height / 2u16;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2)
        )?;

        self.write_centered("Lancement de l'installateur du Modloader")?; //lang
        launch_executable(filepath)
            .log_expect("Error when launching modloader executable");

        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2)
        )?;

        self.write_centered("Lancement terminé...")?; //lang
        sleep(Duration::from_secs(1));
        Ok(())
    }

    /// Page shown when removing modpack files
    pub fn remove_files_page(
        &self,
        base_folderpath: &str,
        folders: &[&str],
    ) -> crossterm::Result<()> {
        let height = self.terminal_height / 2u16;
        let mut stdout = io::stdout();
        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2)
        )?;

        self.write_centered("Suppression des fichiers en cours...")?; //lang*
        folders.iter().for_each(|folder| {
            let folderpath = format!("{}{}", base_folderpath, folder);
            match remove_folder(&folderpath) {
                FileStatus::Ok => {}
                FileStatus::NoChange => {
                    execute!(
                        stdout,
                        terminal::Clear(terminal::ClearType::All),
                        cursor::MoveTo(0, height - 2)
                    )
                    .log_expect("Error when removing modpack files");

                    self.write_centered("Fichier déja supprimé").unwrap(); //lang
                    sleep(Duration::from_millis(250));
                }
                FileStatus::Error(e) => {
                    utils::panic_log(&format!("Error when trying to remove modpack folder: {}", e));
                }
            };
        });
        sleep(Duration::from_millis(600));

        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2)
        )?;

        self.write_centered("Suppression terminée")?; //lang
        sleep(Duration::from_secs(1));
        Ok(())
    }
}
