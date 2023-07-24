use std::process::Command;
use std::path::Path;
use std::fs::{File, create_dir_all};
use std::io::{Write, Read, self};
use std::sync::mpsc;
use ureq;

/// Downloads a file, saves it to the specified path and sends the download status through a channel.
/// the `DownloadStatus::Downloading(f32)` is a float between 0 and 1, representing the percentage of the file downloaded.
/// send `DownloadStatus::Downloaded` when the download is finished.
pub fn download_file(path: &str, url: &str, tx: mpsc::Sender<DownloadStatus>) -> io::Result<()> {
    let mut buffer: Vec<u8> = vec![0; 4096];
    let mut file: File = File::create(path)?;
    
    let response = match ureq::get(url).call() {
        Ok(response) => response,
        Err(err) => {
            tx.send(DownloadStatus::Error(err)).unwrap();
            return Ok(());
        } 
    };
    let length = response.header("Content-Length").unwrap().parse::<f32>().unwrap();
    let mut stream = response.into_reader();
    
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

pub enum DownloadStatus{
    Error (ureq::Error),
    Downloading (f32),
    Downloaded,
} 

#[allow(dead_code)]
pub enum FileStatus {
    FileExists,
    FileDoesntExist,
    FileError,
}

/// Check if a file exists, if not, create it in the path specified.
pub fn create_folder(path: &str) -> FileStatus{
    if !Path::new(path).exists() {
        create_dir_all(path).expect("Couldn't create folder");
        return FileStatus::FileDoesntExist;
    }
    FileStatus::FileExists
}

/// Unzip a file to a folder
/// extracts the `filename` in the `folderpath`
pub fn unzip_file(filename: &str, folderpath: &str) -> Result<(), io::Error> {
    let mut cmd = Command::new("tar");
    cmd.current_dir(&Path::new(folderpath));
    cmd.arg("-xf").arg(filename);

    match cmd.spawn() {
        Ok(mut child) => {
            let status = child.wait().expect("Failed to wait for the commands");
            if status.success() {
                Ok(())
            } else {
                Err(io::Error::new(io::ErrorKind::Other, "Failed to unzip file"))
            }
        }
        Err(err) => Err(err),
    }
}

/// Launch an executable in a new process, used for launching the fabric/forge installer.
pub fn launch_executable(filepath: &str) {
    Command::new(filepath)
        .spawn()
        .expect("Failed to execute process");
}