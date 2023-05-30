use std::collections::HashMap;
use std::sync::{mpsc, Arc};

use std::path::Path;
use std::fs::{create_dir_all, remove_dir_all, File};
use std::net::TcpStream;
use zip::{ZipArchive, result::ZipResult};

use std::io;
use std::io::{Write, Read};

use std::thread;
use std::thread::sleep;
use std::time::Duration;

use std::process::Command;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    style::{Attribute, Print, Color, Stylize, PrintStyledContent, StyledContent},
    execute, terminal, queue, cursor,
};

const MINECRAFT_FOLDER: &str = "%appdata%\\.minecraft\\";

// ---- Config ---- //

#[derive(Debug)]
pub struct Config {
    pub modpack_url: String,
    pub modloader_url: String,
    pub minecraft_folder: String,
    pub magic_installer_folder: String,
}

impl Config {
    pub fn from(config: &str) -> Config {
        let config = Config::parse_hashmap(config, "\n", "=");
        Config {
            modpack_url: config.get("modpack_url").unwrap().to_string(),
            modloader_url: config.get("modloader_url").unwrap().to_string(),
            minecraft_folder: get_env_path(MINECRAFT_FOLDER),
            magic_installer_folder: format!("{}{}", get_env_path(MINECRAFT_FOLDER), "magic_installer\\"),
        }
    }

    fn parse_hashmap(target: &str, entries_separator: &str, key_value_separator: &str) -> HashMap<String, String> {
        let mut result: HashMap<String, String> = HashMap::new();
        let entries = target.split(entries_separator);
        entries.for_each(|e| {
            if let Some((k,v)) = e.split_once(key_value_separator) {
                result.insert(
                    k.trim().to_string(),
                    v.trim().to_string()
                );
            }
        });
        result
    }
}

// ---- File handling ---- //
pub enum FileStatus {
    FileExists,
    FileDoesntExist,
    FileError,
}

//create a function to create a folder if it doesn't exist
pub fn create_folder(path: &str) -> FileStatus{
    if !Path::new(path).exists() {
        create_dir_all(path).expect("Couldn't create folder");
        return FileStatus::FileDoesntExist;
    }
    FileStatus::FileExists
}

pub enum DownloadStatus{
    Downloading (f32),
    Downloaded,
} 

pub fn download_file(path: &str, mut url: &str, tx: mpsc::Sender<DownloadStatus>) -> io::Result<()> {
    (_, url) = url.split_once("//").unwrap();
    let (host, urlpath) = match url.split_once('/') {
        Some((host, urlpath)) => (host, urlpath),
        None => panic!("Invalid url"),
    };

    let mut stream = TcpStream::connect(host)?;
    let request = format!("GET /{} HTTP/1.1\r\nHost: {}\r\n\r\n",urlpath, host);
    stream.write_all(request.as_bytes())?;

    let mut buffer = vec![0; 4096];
    let mut file = File::create(path)?;
    
    let bytes_read = stream.read(&mut buffer)?;
    file.write_all(&buffer[..bytes_read])?;

    let response_str = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut length: usize = 0;
    let (headers, _) = response_str.split_once("\r\n\r\n").unwrap();
    headers.lines()
        .filter(|l| l.starts_with("Content-Length: "))
        .for_each(|line| {
            let (_, length_str) = line.split_once(": ").unwrap();
            length = length_str.parse::<usize>().unwrap();
        });

    loop {
        let bytes_read = stream.read(&mut buffer)?;
        file.write_all(&buffer[..bytes_read])?;
        tx.send(DownloadStatus::Downloading (file.metadata().unwrap().len() as f32 / length as f32)).unwrap();
        if bytes_read == 0 {
            break;
        }
    }
    tx.send(DownloadStatus::Downloaded).unwrap();
    Ok(())
}

pub fn unzip_file(filepath: &str, folderpath: &str) -> ZipResult<()> {
    let file = File::open(filepath).unwrap();
    let mut archive = ZipArchive::new(file)?;
    archive.extract(folderpath)?;
    Ok(())
}

pub fn launch_executable(filepath: &str) {
    Command::new(filepath);
}

pub fn remove_mods(filepath: &str) -> io::Result<()> {
    remove_dir_all(filepath)?;
    Ok(())
}

pub fn get_env_path(path: &str) -> String {
    if path.starts_with('%') {
        let path_splitted: Vec<&str> = path.split('%').collect();
        let var: &str = &path_splitted[1].to_uppercase();
        let path = match std::env::var(var) {
            Ok(path) => path,
            Err(_) => panic!("Environnement variable '{}' not found", var),
        };
        return path + path_splitted[2];
    }
    path.to_string()
}

// ---- UI ---- //

pub enum AppStatus {
    Loop,
    Exit,
}

pub struct Display {
    terminal_width: usize,
    terminal_height: usize,
    config: Config,
}

impl Display {
    pub fn open(config: Config) -> crossterm::Result<Display>{
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(Display {
            terminal_width: terminal::size()?.0 as usize,
            terminal_height: terminal::size()?.1 as usize,
            config
        })
    }

    pub fn close(&self) -> crossterm::Result<()> {
        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
        Ok(())
    }

    fn write_centered(&self, text: &str) -> crossterm::Result<()>{
        let padding: usize = self.terminal_width.saturating_sub(text.len()) / 2;
        execute!(io::stdout(), Print(" ".repeat(padding)), Print(text))?;
        Ok(())
    }

    fn write_stylized_centered(&self, stylized_text: StyledContent<&str>) -> crossterm::Result<()> {
        let padding: usize = self.terminal_width.saturating_sub(stylized_text.content().len()) / 2;
        execute!(io::stdout(), Print(" ".repeat(padding)), PrintStyledContent(stylized_text))?;
        Ok(())
    }

    // MAIN MENU
    pub fn main_menu(&mut self) -> crossterm::Result<AppStatus> {
        let options: [&str; 4] = ["Install modpack", "Install fabric loader", "Remove modpack", "Exit (esc)"];
        let options_len = options.len();

        let mut selected = 0;
        let key_pressed: KeyCode;
        
        // Main drawing
        self.draw_main_menu(selected, &options)?;

        // Event loop
        loop {
            if let Event::Key(KeyEvent {code, kind, ..}) = event::read().unwrap() {
                if kind == KeyEventKind::Press {
                    match code {
                        KeyCode::Up => {
                            selected = (selected-1)%options_len;
                            self.draw_main_options(selected, &options)?;
                        }
                        KeyCode::Down => {
                            selected = (selected+1)%options_len;
                            self.draw_main_options(selected, &options)?;
                        }
                        KeyCode::Enter => {key_pressed = KeyCode::Enter; break}
                        KeyCode::Esc => {key_pressed = KeyCode::Esc; break}
                        _ => {}
                    }
                }
            }
            if let Event::Resize(width, height) = event::read().unwrap() {
                self.terminal_width = width as usize;
                self.terminal_height = height as usize;
                self.draw_main_menu(selected, &options)?;
            }

            thread::sleep(Duration::from_millis(50));
        }

        match key_pressed {
            KeyCode::Esc => {return Ok(AppStatus::Exit)}
            _ => {match selected {
                0 => {
                    let filepath = format!("{}{}", &self.config.magic_installer_folder, "modpack.zip");
                    self.download_page(&filepath, &self.config.modpack_url)?;
                    self.unzip_page(&filepath, &self.config.minecraft_folder)?;
                }
                1 => {
                    let filepath = format!("{}{}", &self.config.magic_installer_folder, "fabric-loader.exe");
                    self.download_page(&filepath, &self.config.modloader_url)?;
                    self.executable_page(&filepath)?;
                }
                2 => {
                    let folders = ["mods", "config"];
                    self.remove_files_page(&self.config.minecraft_folder, &folders)?;
                }
                3 => {return Ok(AppStatus::Exit)}
                _ => {}
            }},
        };
        Ok(AppStatus::Loop)
    }

    fn draw_main_menu(&self, selected: usize, options: &[&str]) -> crossterm::Result<()>{
        let title = include_str!("../src/title.txt");
        let author = "RICHELET Arthur - 2023";
        let subtitle = "Si les touches ne répondent plus, il faut soit relancer le programme avec Ctrl+C";

        let first_line = 100; //title.lines().next().unwrap(); // 100 is the length of the first line of the title
        let padding = self.terminal_width.saturating_sub(first_line) / 2; 
        let mut stdout = io::stdout();

        execute!(stdout, terminal::Clear(terminal::ClearType::All), cursor::Hide)?;
        execute!(stdout, cursor::MoveTo(0, 0))?;
        title.lines().for_each(|line| {
            queue!(stdout, Print(" ".repeat(padding)), PrintStyledContent(line.with(Color::Blue)), Print("\n")).unwrap();
        });
        stdout.flush()?;

        execute!(stdout, cursor::MoveTo(0, 15))?;
        self.write_stylized_centered(author.with(Color::Blue).attribute(Attribute::Dim))?;
        execute!(stdout, cursor::MoveTo(0, 29))?;
        self.write_stylized_centered(subtitle.with(Color::DarkGrey).attribute(Attribute::Dim))?;

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
        let path = Arc::new(path.to_owned());
        let url = Arc::new(url.to_owned());

        let mut stdout = io::stdout();
        let height: u16 = (self.terminal_height as f32 / 2.0) as u16;
        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Téléchargement en cours...")?; //lang
        execute!(stdout, cursor::MoveTo(0, height))?;
        self.write_centered("Préparation du téléchargement")?; //lang

        execute!(stdout, cursor::MoveTo(0, height*2u16))?;
        self.write_stylized_centered("Si le télécharchement semble rester à 0%, Ctrl+C peut débloquer le programme".with(Color::DarkGrey))?; //lang

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
                Err(_) => {}
            }
        }
        handle.join().unwrap();

        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Téléchargement fini !")?; //lang
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

    pub fn unzip_page(&self, filepath: &str, folderpath: &str) -> crossterm::Result<()> {
        let height = self.terminal_height as u16 / 2u16;
        let mut stdout = io::stdout();
        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Installation en cours...")?; //lang
        unzip_file(filepath, folderpath).unwrap();
        
        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Installation terminée...")?; //lang
        sleep(Duration::from_secs(1));
        Ok(())
    }

    pub fn executable_page(&self, filepath: &str) -> crossterm::Result<()> {
        let height = self.terminal_height as u16 / 2u16;
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
        let height = self.terminal_height as u16 / 2u16;
        let mut stdout = io::stdout();
        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Suppression des fichiers en cours...")?; //lang*
        folders.iter().for_each(|folder| {
            let folderpath = format!("{}{}", base_folderpath, folder);
            remove_mods(&folderpath).unwrap();
        });
        
        execute!(stdout,
            terminal::Clear(terminal::ClearType::All),
            cursor::MoveTo(0, height - 2))?;

        self.write_centered("Suppression terminée...")?; //lang
        sleep(Duration::from_secs(1));
        Ok(())
    }
}
