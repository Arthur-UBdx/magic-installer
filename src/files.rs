use zip::{ZipArchive, result::ZipResult};

use std::process::Command;
use std::path::Path;
use std::fs::{File, create_dir_all};
use std::io::{Write, Read, self};
use std::net::TcpStream;
use std::sync::mpsc;
use std::borrow::Cow;


pub fn download_file(path: &str, mut url: &str, tx: mpsc::Sender<DownloadStatus>) -> io::Result<()> {
    (_, url) = url.split_once("//").unwrap();
    let (host, urlpath) = match url.split_once('/') {
        Some((host, urlpath)) => (host, urlpath),
        None => panic!("Invalid url"),
    };

    let mut stream: TcpStream = TcpStream::connect(host)?;
    let request: String = format!("GET /{} HTTP/1.1\r\nHost: {}\r\n\r\n",urlpath, host);
    stream.write_all(request.as_bytes())?;

    let mut buffer: Vec<u8> = vec![0; 4096];
    let mut file: File = File::create(path)?;
    
    let bytes_read: usize = stream.read(&mut buffer)?;
    file.write_all(&buffer[..bytes_read])?;

    let response_str: Cow<'_, str> = String::from_utf8_lossy(&buffer[..bytes_read]);
    let mut length: usize = 0;
    let (headers, _) = response_str.split_once("\r\n\r\n").unwrap();
    headers.lines()
        .filter(|l| l.starts_with("Content-Length: "))
        .for_each(|line| {
            let (_, length_str) = line.split_once(": ").unwrap();
            length = length_str.parse::<usize>().unwrap();
        });

    loop {
        let bytes_read: usize = stream.read(&mut buffer)?;
        file.write_all(&buffer[..bytes_read])?;
        tx.send(DownloadStatus::Downloading (file.metadata().unwrap().len() as f32 / length as f32)).unwrap();
        if bytes_read == 0 {
            break;
        }
    }
    tx.send(DownloadStatus::Downloaded).unwrap();
    Ok(())
}


// ---- File handling ---- //
pub enum FileStatus {
    FileExists,
    FileDoesntExist,
    //FileError,
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

pub fn unzip_file(filepath: &str, folderpath: &str) -> ZipResult<()> {
    let file = File::open(filepath).unwrap();
    let mut archive = ZipArchive::new(file)?;
    archive.extract(folderpath)?;
    Ok(())
}

pub fn launch_executable(filepath: &str) {
    Command::new(filepath)
        .spawn()
        .expect("Failed to execute process");
}