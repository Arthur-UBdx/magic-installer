use crate::modules::config::{
    Config, AUTHOR, BOTTOM_TEXT, CONTROLS, FILES_TO_REMOVE, MAIN_MENU_OPTIONS, MAIN_TITLE, VERSION,
};
use crate::modules::files::{download_file, launch_executable, unzip_file, DownloadStatus};

use std::fs::{remove_dir_all, File};
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

pub enum AppStatus {
    Loop,
    Exit,
}

trait UnwrapOrLog<T, E> {
    fn unwrap_or_log(self, log_file: &mut File) -> T;
}

impl<T, E: std::fmt::Display + std::fmt::Debug> UnwrapOrLog<T, E> for Result<T, E> {
    fn unwrap_or_log(self, log_file: &mut File) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                writeln!(log_file, "Error: {}", error).unwrap();
                panic!("Error: {:?}", error);
            }
        }
    }
}

pub struct Display<'a> {
    terminal_width: u16,
    terminal_height: u16,
    config: &'a Config,
}

impl<'a> Display<'a> {
    /// Creates a new Display instance and enters the alternate screen mode.
    /// This function should be called at the beginning of the application.
    /// It initializes the terminal size and sets up the display.
    pub fn open(config: &'a Config) -> crossterm::Result<Display<'a>> {
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(Display {
            terminal_width: terminal::size()?.0,
            terminal_height: terminal::size()?.1,
            config,
        })
    }

    /// Closes the display and exits the alternate screen mode.
    /// This function should be called when the application is done using the terminal.
    /// It restores the terminal to its original state.
    pub fn close(&self) -> crossterm::Result<()> {
        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
        Ok(())
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
            if event::poll(Duration::from_millis(100))? {
                if let Event::Key(KeyEvent { code, .. }) = event::read().unwrap() {
                    match code {
                        KeyCode::Up => {
                            selected = (selected - 1) % options_len;
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
                if let Event::Resize(width, height) = event::read().unwrap() {
                    self.terminal_width = width;
                    self.terminal_height = height;
                    self.draw_main_menu(selected, options)?;
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
                        let folders: &[&str] = FILES_TO_REMOVE;

                        self.config
                            .log(format!("modpack zip file path: {}", &filepath).as_str());
                        self.config
                            .log(format!("files to remove path: {:?}", &folders).as_str());

                        self.remove_files_page(&self.config.minecraft_folder, folders)?;
                        self.download_page(&filepath, &self.config.modpack_url)
                            .unwrap_or_log(&mut self.config.debugfile);
                        self.unzip_page(filename, &self.config.minecraft_folder)
                            .unwrap_or_log(&mut self.config.debugfile);
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

                        self.config
                            .log(format!("modloader zip path: {}", &filepath).as_str());
                        self.config
                            .log(format!("modloader exec path: {}", &executable_path).as_str());
                        self.config.log(
                            format!(
                                "magic_installer folder path: {}",
                                &self.config.magic_installer_folder
                            )
                            .as_str(),
                        );

                        self.download_page(&filepath, &self.config.modloader_url)
                            .unwrap_or_log(&mut self.config.debugfile);
                        self.unzip_page(filename, &self.config.magic_installer_folder)
                            .unwrap_or_log(&mut self.config.debugfile);
                        self.executable_page(&executable_path)
                            .unwrap_or_log(&mut self.config.debugfile);
                    }
                    2 => {
                        // remove all files
                        let folders = FILES_TO_REMOVE;
                        self.remove_files_page(&self.config.minecraft_folder, folders)?;
                    } // exit
                    3 => return Ok(AppStatus::Exit),
                    _ => {}
                }
            }
        };
        Ok(AppStatus::Loop)
    }

    /// Draws the main menu of the application.
    /// This function takes the selected index and the options to display.
    /// It uses the `crossterm` library to handle terminal output and styling.
    /// It clears the current line and writes the title, author, controls, and bottom text.
    /// The selected option is highlighted with a blue color and bold attribute.
    /// It uses the `execute!` macro to perform terminal operations.
    /// It returns a `crossterm::Result<()>` indicating success or failure.
    /// This function is useful for displaying the main menu of the application.
    /// It also handles the terminal size and padding for centering the text.
    fn draw_main_menu(&self, selected: usize, options: &[&str]) -> crossterm::Result<()> {
        let title: &str = MAIN_TITLE;
        let author: String = format!("{} - {}", AUTHOR, VERSION);
        let bottom_text: &str = BOTTOM_TEXT;
        let controls: &str = CONTROLS;

        let first_line = 100; //title.lines().next().unwrap(); // 100 is the length of the first line of the title
        let padding = (self.terminal_width.saturating_sub(first_line) / 2) as usize;
        let mut stdout = io::stdout();

        execute!(
            stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::Hide
        )?;
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
    /// It uses the `crossterm` library to handle terminal output and styling.
    /// It clears the current line and writes the options to the terminal.
    /// The selected option is highlighted with a green color and bold attribute.
    /// It uses the `execute!` macro to perform terminal operations.
    /// It returns a `crossterm::Result<()>` indicating success or failure.
    /// This function is useful for displaying a list of options in the main menu.
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
        launch_executable(filepath).unwrap_or_log(&mut self.config.debugfile);

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
            match remove_dir_all(folderpath) {
                Ok(()) => {}
                Err(_) => {
                    execute!(
                        stdout,
                        terminal::Clear(terminal::ClearType::All),
                        cursor::MoveTo(0, height - 2)
                    )
                    .unwrap();

                    self.write_centered("Fichier déja supprimé").unwrap(); //lang
                    sleep(Duration::from_millis(250));
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
