use std::collections::HashMap;
use std::sync::mpsc;

use std::path::Path;
use std::fs::{create_dir_all, File};
use std::net::TcpStream;

use std::io;
use std::io::{Write, Read};

use std::thread;
use std::time::Duration;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    style::{self, Attribute, Print, Color, Stylize, PrintStyledContent, StyledContent},
    execute, terminal, queue, cursor,
};

// ---- Config ---- //

#[derive(Debug)]
pub struct Config {
    pub modpack_url: String,
    pub modloader_url: String,
}

impl Config {
    pub fn from_str(config: &str) -> Config {
        let config = Config::parse_hashmap(config, "\n", "=");
        Config {
            modpack_url: config.get("modpack_url").unwrap().to_string(),
            modloader_url: config.get("modloader_url").unwrap().to_string(),
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

pub fn download_file(path: &str, mut url: &str, tx: ) -> io::Result<()> {
    (_, url) = url.split_once("//").unwrap();
    let (host, urlpath) = match url.split_once("/") {
        Some((host, urlpath)) => (host, urlpath),
        None => panic!("Invalid url"),
    };

    let mut stream = TcpStream::connect(host)?;
    let request = format!("GET /{} HTTP/1.1\r\nHost: {}\r\n\r\n",urlpath, host);
    println!("{}", request);
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
        println!("{} / {}", file.metadata().unwrap().len(), length);
        if bytes_read == 0 {
            break;
        }
    }
    Ok(())
}


pub fn get_env_path(path: &str) -> String {
    if path.starts_with("%") {
        let path_splitted: Vec<&str> = path.split("%").collect();
        let var: &str = &format!("{}", &path_splitted[1].to_uppercase());
        let path = match std::env::var(var) {
            Ok(path) => path,
            Err(_) => panic!("Environnement variable '{}' not found", var),
        };
        return path.to_string() + &path_splitted[2].to_string();
    }
    path.to_string()
}

// ---- UI ---- //

pub struct Display {
    terminal_width: usize,
    terminal_height: usize,
}

impl Display {
    pub fn open() -> crossterm::Result<Display>{
        execute!(io::stdout(), terminal::EnterAlternateScreen, cursor::Hide)?;
        Ok(Display {
            terminal_width: terminal::size()?.0 as usize,
            terminal_height: terminal::size()?.1 as usize,
        })
    }

    pub fn close(&self) -> crossterm::Result<()> {
        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;
        Ok(())
    }

    fn write_centered(&self, text: &str) -> crossterm::Result<()>{
        let padding: usize = self.terminal_width.checked_sub(text.len()).unwrap_or(0) / 2;
        execute!(io::stdout(), Print(" ".repeat(padding)), Print(text))?;
        Ok(())
    }

    fn write_stylized_centered(&self, stylized_text: StyledContent<&str>) -> crossterm::Result<()> {
        let padding: usize = self.terminal_width.checked_sub(stylized_text.content().len()).unwrap_or(0) / 2;
        execute!(io::stdout(), Print(" ".repeat(padding)), PrintStyledContent(stylized_text))?;
        Ok(())
    }

    // MAIN MENU
    pub fn main_menu(&mut self) -> crossterm::Result<()> {
        let options: [&str; 4] = ["Install modpack", "Install fabric loader", "Remove modpack", "Exit (esc)"];
        let options_len = options.len();

        let mut selected = 0;
        let key_pressed: KeyCode;
        
        // Main drawing
        self.draw_main_menu(selected, &options)?;

        // Event loop
        loop {
            if let Event::Key(KeyEvent {code, kind, ..}) = event::read().unwrap() {
                match kind {
                    KeyEventKind::Press => {
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
                    _ => {}
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
            KeyCode::Esc => {return Ok(())}
            _ => {match selected {
                0 => {println!("Install modpack")}
                1 => {println!("Install fabric loader")}
                2 => {println!("Remove modpack")}
                3 => {return Ok(())}
                _ => {}
            }},
        };
        Ok(())
    }

    fn draw_main_menu(&self, selected: usize, options: &[&str]) -> crossterm::Result<()>{
        let title = include_str!("../src/title.txt");
        let author = "RICHELET Arthur - 2023";
        let subtitle = "Il faut relancer le programme avec Ctrl+C si les touches ne répondent plus.";

        let first_line = 100; //title.lines().next().unwrap(); // 100 is the length of the first line of the title
        let padding = self.terminal_width.checked_sub(first_line).unwrap_or(0) / 2; 
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



    

}
