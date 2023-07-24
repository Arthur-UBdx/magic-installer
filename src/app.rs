use crate::files::{launch_executable, download_file, unzip_file, DownloadStatus};
use crate::config::{VERSION, MAIN_TITLE, AUTHOR, CONTROLS, BOTTOM_TEXT, MAIN_MENU_OPTIONS, FILES_TO_REMOVE, Config};

use std::thread;
use std::thread::sleep;
use std::time::Duration;
use std::fs::{File, remove_dir_all};
use std::sync::Arc;
use std::io::{Write, self};

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent},
    style::{Attribute, Print, Color, Stylize, PrintStyledContent, StyledContent},
    execute, terminal, queue, cursor,
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

pub struct Display {
    terminal_width: u16,
    terminal_height: u16,
    config: Config,
}

impl Display {
    pub fn open(config: Config) -> crossterm::Result<Display>{
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(Display {
            terminal_width: terminal::size()?.0,
            terminal_height: terminal::size()?.1,
            config
        })
    }

    pub fn close(&self) -> crossterm::Result<()> {
        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
        Ok(())
    }

    fn write_centered(&self, text: &str) -> crossterm::Result<()>{
        let padding: usize = (self.terminal_width.saturating_sub(text.len() as u16) / 2) as usize;
        execute!(io::stdout(), Print(" ".repeat(padding)), Print(text))?;
        Ok(())
    }

    fn write_stylized_centered(&self, stylized_text: StyledContent<&str>) -> crossterm::Result<()> {
        let padding: usize = (self.terminal_width.saturating_sub(stylized_text.content().len() as u16) / 2) as usize;
        execute!(io::stdout(), Print(" ".repeat(padding)), PrintStyledContent(stylized_text))?;
        Ok(())
    }

    // MAIN MENU
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
                if let Event::Key(KeyEvent {code, ..}) = event::read().unwrap() {
                    match code {
                        KeyCode::Up => {
                            selected = (selected-1)%options_len;
                            self.draw_main_options(selected, options)?;
                        }
                        KeyCode::Down => {
                            selected = (selected+1)%options_len;
                            self.draw_main_options(selected, options)?;
                        }
                        KeyCode::Enter => {key_pressed = KeyCode::Enter; break}
                        KeyCode::Esc => {key_pressed = KeyCode::Esc; break}
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

        match key_pressed {
            KeyCode::Esc => {return Ok(AppStatus::Exit)}
            _ => {match selected {
                0 => { // install the modpack
                    let filename: &str = "modpack.zip";
                    let filepath: String = format!("{}{}", &self.config.minecraft_folder, filename);
                    let folders: &[&str] = FILES_TO_REMOVE;

                    self.config.log(format!("modpack zip file path: {}", &filepath).as_str());
                    self.config.log(format!("files to remove path: {:?}", &folders).as_str());
                    
                    self.remove_files_page(&self.config.minecraft_folder, folders)?;
                    self.download_page(&filepath, &self.config.modpack_url).unwrap_or_log(&mut self.config.debugfile);
                    self.unzip_page(&filename, &self.config.minecraft_folder).unwrap_or_log(&mut self.config.debugfile);
                }
                1 => { // install the modloader (fabric/forge)
                    let filename: &str = "modloader.zip";
                    let filepath: String = format!("{}{}", &self.config.magic_installer_folder, filename);
                    let executable_path: String = format!("{}{}", &self.config.magic_installer_folder, self.config.modloader_execname);
                    
                    self.config.log(format!("modloader zip path: {}", &filepath).as_str());
                    self.config.log(format!("modloader exec path: {}", &executable_path).as_str());
                    self.config.log(format!("magic_installer folder path: {}", &self.config.magic_installer_folder).as_str());

                    self.download_page(&filepath, &self.config.modloader_url).unwrap_or_log(&mut self.config.debugfile);
                    self.unzip_page(&filename, &self.config.magic_installer_folder).unwrap_or_log(&mut self.config.debugfile);
                    self.executable_page(&executable_path).unwrap_or_log(&mut self.config.debugfile);
                }
                2 => { // remove all files
                    let folders = FILES_TO_REMOVE;
                    self.remove_files_page(&self.config.minecraft_folder, folders)?;
                } // exit
                3 => {return Ok(AppStatus::Exit)}
                _ => {}
            }},
        };
        Ok(AppStatus::Loop)
    }

    fn draw_main_menu(&self, selected: usize, options: &[&str]) -> crossterm::Result<()>{
        let title: &str = MAIN_TITLE;
        let author: String = format!("{} - {}", AUTHOR, VERSION);
        let bottom_text: &str = BOTTOM_TEXT;
        let controls: &str = CONTROLS;

        let first_line = 100; //title.lines().next().unwrap(); // 100 is the length of the first line of the title
        let padding = (self.terminal_width.saturating_sub(first_line) / 2) as usize; 
        let mut stdout = io::stdout();

        execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::Hide)?;
        execute!(stdout, cursor::MoveTo(0, 0))?;
        title.lines().for_each(|line| {
            queue!(stdout, Print(" ".repeat(padding)), PrintStyledContent(line.with(Color::Blue)), Print("\n")).unwrap();
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

    pub fn draw_main_options(&self, selected: usize, options: &[&str]) -> crossterm::Result<()> {
        let mut stdout = io::stdout();
        execute!(stdout, cursor::MoveTo(0, 4))?;
        options.iter().enumerate().for_each(|(index, option)| {
            execute!(stdout, cursor::MoveTo(0, 20 + 2*index as u16)).unwrap();
            execute!(stdout, terminal::Clear(terminal::ClearType::CurrentLine)).unwrap();
            if index == selected {
                self.write_stylized_centered(format!("> {}", option).as_str().with(Color::Green).attribute(Attribute::Bold)).unwrap();
            } else {
                self.write_centered(option).unwrap();
            }
        });
        stdout.flush()?;
        Ok(())
    }


    // Téléchargement et Installation
    pub fn download_page(&self, path: &str, url: &str) -> crossterm::Result<()> {
        let path: Arc<String> = Arc::new(path.to_owned());
        let url: Arc<String> = Arc::new(url.to_owned());

        let mut stdout: io::Stdout = io::stdout();
        let height: u16 = (self.terminal_height as f32 / 2.0) as u16;
        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

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
                    self.write_centered(&format!("{} {}%", Display::download_bar(percentage), (percentage*100.0) as u32))?;
                },
                Ok(DownloadStatus::Downloaded) => {
                    break;
                },
                Ok(DownloadStatus::Error(error)) => {
                    execute!(stdout, cursor::MoveTo(0, height))?;
                    self.write_stylized_centered(format!("Erreur: {}", error).as_str().with(Color::Red).attribute(Attribute::Bold)).unwrap();
                    sleep(Duration::from_secs(2));
                    execute!(stdout,terminal::Clear(terminal::ClearType::All))?;
                    execute!(stdout, cursor::MoveTo(0, height))?;
                    return Err(io::Error::new(io::ErrorKind::Other, "Download Error"));
                } 
                Err(_) => {}
            }
        }
        handle.join().unwrap();

        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::Hide,
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Téléchargement terminé !")?; //lang
        sleep(Duration::from_secs(1));
        Ok(())
    }

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

    pub fn unzip_page(&self, filename: &str, folderpath: &str) -> crossterm::Result<()> {
        let height = self.terminal_height / 2u16;
        let mut stdout = io::stdout();
        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Installation en cours...")?; //lang
        unzip_file(filename, folderpath)?;
        
        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Installation terminée...")?; //lang
        Ok(())
    }

    pub fn executable_page(&self, filepath: &str) -> crossterm::Result<()> {
        let height = self.terminal_height / 2u16;
        let mut stdout = io::stdout();
        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Lancement de l'installateur Fabric")?; //lang
        launch_executable(filepath);
        
        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Lancement terminé...")?; //lang
        sleep(Duration::from_secs(1));
        Ok(())
    }

    pub fn remove_files_page(&self, base_folderpath: &str, folders: &[&str]) -> crossterm::Result<()> {
        let height = self.terminal_height / 2u16;
        let mut stdout = io::stdout();
        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Suppression des fichiers en cours...")?; //lang*
        folders.iter().for_each(|folder| {
            let folderpath = format!("{}{}", base_folderpath, folder);
            match remove_dir_all(folderpath) {
                Ok(()) => {},
                Err(_) => {
                    execute!(stdout,
                        terminal::Clear(terminal::ClearType::All),
                        cursor::MoveTo(0, height - 2)).unwrap();
            
                    self.write_centered("Fichier déja supprimé").unwrap(); //lang
                    sleep(Duration::from_millis(250));
                }
            };
        });
        sleep(Duration::from_millis(600));

        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Suppression terminée")?; //lang
        sleep(Duration::from_secs(1));
        Ok(())
    }
}
