use crate::modules::config;
use std::fmt;
use std::fs::{create_dir_all, File};
use std::io::{self, Read, Write};
use std::path::Path;
use std::process::Command;
use std::sync::mpsc;

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- ## SP-PT-001                                                  //
//%--                                                               //
//%-- L'utilitaire doit être capable de fonctionner sur un système  //
//%-- d'exploitation Windows                                        //
//%-- et Linux.                                                     //
//%--                                                               //
//%-----------------------------------------------------------------//
const WIN_VARIABLES_REGEX: &str = r"%([A-z])%";
const LINUX_VARIABLES_REGEX: &str = r"\$([A-z])+";

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- ## SP-FN-002                                                  //
//%--                                                               //
//%-- Lors du choix de l'action "Installer/mettre à jour le         //
//%-- modpack", l'utilitaire doit                                   //
//%--                                                               //
//%-- 1. Télécharger le modpack depuis un serveur distant dont      //
//%-- l'adresse est spécifiée dans le                               //
//%-- fichier de configuration tout en rendant compte à             //
//%-- l'utilisateur à travers l'interface.                          //
//%--                                                               //
//%-----------------------------------------------------------------//
pub enum DownloadStatus {
    Error(Box<ureq::Error>),
    Downloading(f32),
    Downloaded,
}

/// Downloads a file, saves it to the specified path and sends the download status through a channel.
/// the `DownloadStatus::Downloading(f32)` is a float between 0 and 1, representing the percentage of the file downloaded.
/// send `DownloadStatus::Downloaded` when the download is finished.
pub fn download_file(path: &str, url: &str, tx: mpsc::Sender<DownloadStatus>) -> io::Result<()> {
    let mut buffer: Vec<u8> = vec![0; 4096];
    let mut file: File = File::create(path)?;

    let response = match ureq::get(url).call() {
        Ok(response) => response,
        Err(err) => {
            tx.send(DownloadStatus::Error(Box::new(err))).unwrap();
            return Ok(());
        }
    };
    let length = response
        .header("Content-Length")
        .unwrap()
        .parse::<f32>()
        .unwrap();
    let mut stream = response.into_reader();

    loop {
        let bytes_read: usize = stream.read(&mut buffer)?;
        file.write_all(&buffer[..bytes_read])?;
        tx.send(DownloadStatus::Downloading(
            file.metadata().unwrap().len() as f32 / length as f32,
        ))
        .unwrap();
        if bytes_read == 0 {
            break;
        }
    }
    tx.send(DownloadStatus::Downloaded).unwrap();
    Ok(())
}

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- DERIVED:                                                      //
//%--                                                               //
//%-- Creates a folder if it doesn't exists at the specified path   //
//%-- Returns an error if the folder can't be created               //
//%--                                                               //
//%-----------------------------------------------------------------//
#[allow(dead_code)]
pub enum FileStatus {
    DoesntExists,
    Exists,
    Error,
}

/// Check if a file exists, if not, create it in the path specified.
pub fn create_folder_if_not_exists(path: &str) -> FileStatus {
    if !Path::new(path).exists() {
        let folder = create_dir_all(path);
        match folder {
            Ok(_) => return FileStatus::DoesntExists,
            Err(_) => return FileStatus::Error,
        }
    }
    FileStatus::Exists
}
//%-----------------------------------------------------------------//
//%--                                                               //
//%-- DERIVED:                                                      //
//%--                                                               //
//%-- Creates a file if it doesn't exists at the specified path     //
//%-- Returns an error if the file can't be created                 //
//%--                                                               //
//%-----------------------------------------------------------------//

/// Check if a file exists, if not, create it in the path specified.
pub fn create_file_if_not_exists(path: &str) -> FileStatus {
    if !Path::new(path).exists() {
        let file = File::create(path);
        match file {
            Ok(_) => return FileStatus::DoesntExists,
            Err(_) => return FileStatus::Error,
        }
    }
    FileStatus::Exists
}
//%-----------------------------------------------------------------//
//%--                                                               //
//%-- DERIVED:                                                      //
//%--                                                               //
//%-- Reads a file and returns its content as a string              //
//%--                                                               //
//%-----------------------------------------------------------------//
/// Read a file and return its content as a string
/// the file is specified by the path
pub fn read_file(path: &str) -> Result<String, io::Error> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}

/// Appends to a file, if the file doesn't exists, it will be created
/// the file is specified by the path
pub fn append_to_file(path: &str, content: &str) -> Result<(), io::Error> {
    let mut file = File::options().append(true).create(true).open(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- DERIVED:                                                      //
//%--                                                               //
//%-- Takes a zip filename in input and a folder path, extracts the //
//%-- zip file in the folder path.                                  //
//%--                                                               //
//%-----------------------------------------------------------------//
/// Unzip a file to a folder
/// extracts the `filename` in the `folderpath`
pub fn unzip_file(filename: &str, folderpath: &str) -> Result<(), io::Error> {
    let mut cmd = Command::new("tar");
    let folder_path = Path::new(folderpath);
    cmd.current_dir(folder_path);
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

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- #DERIVED:                                                     //
//%-- Removes a folder, returns FileStatus::Error if the folder     //
//%-- can't be                                                      //
//%-- removed,                                                      //
//%-- FileStatus::DoesntExists if the file doesn't exists and       //
//%-- FileStatus::Exists if the file was removed                    //
//%--                                                               //
//%-----------------------------------------------------------------//
/// Remove a folder, returns FileStatus::Error if the folder can't be removed,
/// FileStatus::DoesntExists if the file doesn't exists and
/// FileStatus::Exists if the file was removed
/// the folder is specified by the path
pub fn remove_folder(path: &str) -> FileStatus {
    if Path::new(path).exists() {
        let folder = std::fs::remove_dir_all(path);
        match folder {
            Ok(_) => return FileStatus::Exists,
            Err(_) => return FileStatus::Error,
        }
    }
    FileStatus::DoesntExists
}

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- ## SP-FN-003                                                  //
//%--                                                               //
//%-- Lors du choix de l'action "Installer le modloader",           //
//%-- l'utilitaire doit                                             //
//%--                                                               //
//%-- 3. Lancer l'éxécutable d'installation du modloader            //
//%--                                                               //
//%-----------------------------------------------------------------//
/// The error that can occur when launching an executable.
#[derive(Debug)]
pub enum ExecutableError {
    UnspecifiedError,
    ExecutableNotFound,
    ExecutableNotSupported,
}

impl fmt::Display for ExecutableError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ExecutableError::UnspecifiedError => write!(f, "An unspecified error occurred."),
            ExecutableError::ExecutableNotFound => write!(f, "Executable not found."),
            ExecutableError::ExecutableNotSupported => write!(f, "Executable not supported."),
        }
    }
}
//-------------//

/// The type of the modloader executable, either a jar or an executable (.exe).
enum ModloaderExecutableType {
    Jar,
    Executable,
}

impl ModloaderExecutableType {
    /// Get the type of the modloader executable from the filename.
    fn from_filename(filename: &str) -> Result<ModloaderExecutableType, ExecutableError> {
        if filename.ends_with(".jar") {
            return Ok(ModloaderExecutableType::Jar);
        } else if filename.ends_with(".exe") {
            return Ok(ModloaderExecutableType::Executable);
        }
        Err(ExecutableError::ExecutableNotSupported)
    }
}
//-------------//

/// Launch an executable in a new process, used for launching the fabric/forge installer.
pub fn launch_executable(filepath: &str) -> Result<(), ExecutableError> {
    let exec_type = ModloaderExecutableType::from_filename(filepath)?;
    match exec_type {
        ModloaderExecutableType::Jar => launch_executable_jar(filepath, vec![]),
        ModloaderExecutableType::Executable => launch_executable_exec(filepath),
    }
}

/// Launch an executable in a new process, used for launching the fabric/forge installer.
/// This function is used when the modloader is an executable (.exe).
fn launch_executable_exec(filepath: &str) -> Result<(), ExecutableError> {
    if !Path::new(filepath).exists() {
        return Err(ExecutableError::ExecutableNotFound);
    }

    match Command::new(filepath).spawn() {
        Ok(_) => Ok(()),
        Err(_) => Err(ExecutableError::UnspecifiedError),
    }
}

/// Launch an executable jar in a new process, used for launching the fabric/forge installer.
fn launch_executable_jar(filepath: &str, args: Vec<&str>) -> Result<(), ExecutableError> {
    if !Path::new(filepath).exists() {
        return Err(ExecutableError::ExecutableNotFound);
    }

    let mut cmd = Command::new("java");
    cmd.arg("-jar").arg(filepath);
    args.iter().for_each(|arg| {
        cmd.arg(arg);
    });

    match cmd.spawn() {
        Ok(_) => Ok(()),
        Err(_) => Err(ExecutableError::UnspecifiedError),
    }
}
//%-----------------------------------------------------------------//
//%-- End of SP-FN-003                                              //
//%-----------------------------------------------------------------//

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- DERIVED:                                                      //
//%--                                                               //
//%-- Expands the variables in a path string, bash style.           //
//%-- Variables like %APPDATA% or $HOME will be replaced by their   //
//%-- values. The result will be returned in a string               //
//%--                                                               //
//%-----------------------------------------------------------------//
/// Expand the variables in a path string, bash style.
/// Variables like %APPDATA% or $HOME will be replaced by their values.
/// The function will return the expanded path as a string.
#[allow(unreachable_patterns)]
pub fn expand_variables(path: String) -> String {
    match config::OS_TYPE {
        config::OSType::Windows => {
            // captures the variables in the path string
            let caps = match regex::Regex::new(WIN_VARIABLES_REGEX) {
                Ok(regex) => regex,
                Err(e) => panic_log(format!("Error creating windows regex: {}", e)),
            };
            // clones the path to modify it
            let mut expanded_path = path.clone();
            // for each capture, get the variable name and replace it with its value
            for cap in caps.captures_iter(&path) {
                let var = cap.get(1).unwrap().as_str();
                let value = std::env::var(var).unwrap_or_default();
                expanded_path = expanded_path.replace(&cap[0], &value);
            }
            expanded_path
        }
        config::OSType::Linux => {
            let caps = match regex::Regex::new(LINUX_VARIABLES_REGEX) {
                Ok(regex) => regex,
                Err(e) => panic_log(format!("Error creating linux regex: {}", e)),
            };
            // clones the path to modify it
            let mut expanded_path = path.clone();
            // for each capture, get the variable name and replace it with its value
            for cap in caps.captures_iter(&path) {
                let var = cap.get(1).unwrap().as_str();
                let value = std::env::var(var).unwrap_or_default();
                expanded_path = expanded_path.replace(&cap[0], &value);
            }
            expanded_path
        }
        // unreachable pattern but considered for safety in case more OS were added
        _ => {
            panic_log(format!("OS not supported"));
        }
    }
}

/// Log a message to the debug file.
pub fn log(message: &str) -> () {
    append_to_file(
        format!(
            "{}{}\n",
            config::get_minecraft_folder(),
            "magic_installer/debug.txt"
        )
        .as_str(),
        message,
    )
    .unwrap_or_else(|_| {
        panic!("Impossible d'écrire dans le fichier de log : {}", message);
    });
}

/// Panics and logs the message to the debug file.
pub fn panic_log(message: String) -> ! {
    log(&message);
    panic!("{}", &message);
}
