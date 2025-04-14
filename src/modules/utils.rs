#[allow(unused_imports)]
use crate::modules::{app, config, files};

use std::collections::HashMap;
//%-----------------------------------------------------------------//
//%--                                                               //
//%-- ## SP-PT-001                                                  //
//%--                                                               //
//%-- L'utilitaire doit être capable de fonctionner sur un système  //
//%-- d'exploitation Windows                                        //
//%-- et Linux.                                                     //
//%--                                                               //
//%-----------------------------------------------------------------//

#[cfg(target_os = "windows")]
const WIN_VARIABLES_REGEX: &str = r"%([A-z]+)%";
#[cfg(target_os = "linux")]
const LINUX_VARIABLES_REGEX: &str = r"\$([A-z]+)";

/// This trait is used to unwrap a Result and log the error if it occurs.
pub trait LogExcept<T> {
    fn log_expect(self, message: &str) -> T;
}

/// It is a custom implementation of the `unwrap_or` method for Result types.
/// It takes a Result<T, E> and returns T if the result is Ok.
/// If the result is an Err, it logs the error message and panics.
/// This is useful for handling errors in a consistent way throughout the application.
impl<T, E: std::fmt::Debug> LogExcept<T> for Result<T, E> {
    fn log_expect(self, message: &str) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                panic_log(&format!("Error: {:?}\n\nMessage: {}", error, message));
                // panic!();
            }
        }
    }
}

/// This trait is used to unwrap an Option and log an error message if it is None.
/// It is a custom implementation of the `unwrap_or` method for Option types.
/// It takes an Option<T> and returns T if the option is Some.
/// If the option is None, it logs an error message and panics.
/// This is useful for handling None values in a consistent way throughout the application.
/// It is similar to the `LogExcept` trait for Result types.
impl<T> LogExcept<T> for Option<T> {
    fn log_expect(self, message: &str) -> T {
        match self {
            Some(value) => value,
            _ => {
                panic_log(&format!(
                    "Error: None value unwrapped\nmessage: {}",
                    message
                ));
                // panic!();
            }
        }
    }
}

/// Log a message to the debug file.
pub fn log(message: &str) -> Result<(), std::io::Error> {
    let filepath: String;
    // if the debug mode is enabled, print the message to the console
    if cfg!(test) {
        filepath = String::from("tests/results/tests.log")
    } else {
        match files::create_folder_if_not_exists(&format!(
            "{}magic_installer/logs",
            config::get_minecraft_folder()
        )) {
            files::FileStatus::Error(e) => {
                return Err(e);
            }
            _ => {}
        }
        filepath = format!(
            "{}magic_installer/logs/log-{}.txt",
            config::get_minecraft_folder(),
            format_date()
        );
    }

    files::append_to_file(&filepath, &format!("{}: {}\n", format_date(), message))?;
    Ok(())
}

/// Panics and logs the message to the debug file.
pub fn panic_log(message: &str) -> ! {
    log(message).unwrap_or_else(|e| {
        panic!(
            "An error occured when trying to log the error message : {}\nmessage: {}",
            e, message
        );
    });
    panic!("{}", message);
}

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
    #[cfg(target_os = "windows")]
    {
        // captures the variables in the path string
        let caps = match regex::Regex::new(WIN_VARIABLES_REGEX) {
            Ok(regex) => regex,
            Err(e) => panic_log(&format!("Error creating windows regex: {}", e)),
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
    #[cfg(target_os = "linux")]
    {
        let caps = match regex::Regex::new(LINUX_VARIABLES_REGEX) {
            Ok(regex) => regex,
            Err(e) => panic_log(&format!("Error creating linux regex: {}", e)),
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
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        panic!("Unsupported OS");
    }
}

//%-----------------------------------------------------------------//
//%-- # DERIVED:                                                    //
//%--                                                               //
//%-- Formats the date as a string in the format YYYY-MM-DD_HH:MM:SS//
//%--                                                               //
//%-----------------------------------------------------------------//
/// Formats the date as a string in the format YYYY-MM-DD_HH:MM:SS
/// The function will return the formatted date as a string.
/// This function is used to create a timestamp for the log file.
fn format_date() -> String {
    let now = chrono::Local::now();
    let date = now.format("%Y-%m-%d_%H-%M-%S").to_string();
    date
}
//%-----------------------------------------------------------------//
//%--                                                               //
//%-- # DERIVED:                                                    //
//%--                                                               //
//%-- Remove the comments in a config file, the comments are marked //
//%-- at the beginning of the line by a #                           //
//%--                                                               //
//%-----------------------------------------------------------------//
/// Remove the comments in a config file, the comments are marked
/// at the beginning of the line by a #
/// The function will return a string with the comments removed.
pub fn remove_comments(file: String) -> String {
    let mut result = String::new();
    for line in file.lines() {
        // if the line is empty or starts with a #, skip it
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        // else add the line to the result
        result.push_str(line);
        result.push('\n');
    }
    result
}
//%-----------------------------------------------------------------//
//%--                                                               //
//%-- # DERIVED:                                                    //
//%--                                                               //
//%-- Parses a string into a HashMap using a separator and a        //
//%-- key-value separator.                                          //
//%--                                                               //
//%-----------------------------------------------------------------//
/// Parses a string into a HashMap using a separator and a key-value separator.
/// The function will return a HashMap<String, String> with the key-value pairs.
pub fn parse_hashmap(
    target: &str,
    entries_separator: &str,
    key_value_separator: &str,
) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();
    let entries = target.split(entries_separator);
    entries.for_each(|e| {
        if let Some((k, v)) = e.split_once(key_value_separator) {
            result.insert(k.trim().to_string(), v.trim().to_string());
        }
    });
    result
}

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- # UNIT TESTS:                                                 //
//%--                                                               //
//%-----------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use crate::modules::utils::LogExcept;
    #[allow(unused_imports)]
    use crate::modules::{config, files, utils};
    use std::{collections::HashMap, env};

    //%-----------------------------------------------------------------//
    //%--                                                               //
    //%-- # UNIT TEST: Expand variables                                 //
    //%--                                                               //
    //%-- Tests the expand_variables function to ensure it correctly    //
    //%-- expands                                                       //
    //%-- environment variables in the given path string.               //
    //%--                                                               //
    //%-- The test sets the HOME environment variable explicitly for the//
    //%-- test                                                          //
    //%-- and check if the functions correctly expands the variable in  //
    //%-- '$HOME/magic_installer'                                       //
    //%-- and '%HOME%/magic_installer' to '/test/home/magic_installer'. //
    //%-- The test should be run for each OS type (Windows and Linux) to//
    //%-- ensure                                                        //
    //%-- compatibility across platforms.                               //
    //%--                                                               //
    //%-- Success conditions:                                           //
    //%--   - The path '$HOME/magic_installer' is expanded to           //
    //%-- '/test/home/magic_installer'                                  //
    //%--   - The programm doesn't panic                                //
    //%--                                                               //
    //%-- Failure conditions:                                           //
    //%--   - The path '$HOME/magic_installer' is not equal to          //
    //%-- '/test/home/magic_installer'                                  //
    //%--   - The programm panics during the execution of the test      //
    //%--                                                               //
    //%-----------------------------------------------------------------//
    #[test]
    fn test_expand_variables() {
        // Set the HOME environment variable explicitly for the test
        // The tests have to be run for each OS type
        env::set_var("HOME", "/test/home");
        let input: String;
        #[cfg(target_os = "windows")]
        {
            input = String::from("%HOME%/magic_installer");
        }
        #[cfg(target_os = "linux")]
        {
            input = String::from("$HOME/magic_installer");
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux")))]
        {
            panic!("Unsupported OS");
        }

        let expected_output = String::from("/test/home/magic_installer");
        let result = utils::expand_variables(input);
        assert_eq!(result, expected_output);
    }

    //%-----------------------------------------------------------------//
    //%--                                                               //
    //%-- # UNIT TEST: Log message                                      //
    //%--                                                               //
    //%-- Tests the log function to ensure it correctly logs a message  //
    //%-- to the                                                        //
    //%-- log file.                                                     //
    //%--                                                               //
    //%-- The tests logs a message to the log file, reads it back and   //
    //%-- checks                                                        //
    //%-- if the message is present in the log file.                    //
    //%--                                                               //
    //%-- Success conditions:                                           //
    //%--     - The log file is created if it doesn't exist AND         //
    //%--     - The message is present in the log file AND              //
    //%--     - The programm doesn't panic                              //
    //%--                                                               //
    //%-- Failure conditions:                                           //
    //%--     - The log file is not created OR                          //
    //%--     - The message is not present in the log file OR           //
    //%--     - The programm panics during the execution of the test    //
    //%--                                                               //
    //%-----------------------------------------------------------------//
    #[test]
    fn test_log() {
        let message = "This is a test log message";
        let log_filepath = format!(
            "{}/tests/results/tests.log",
            std::env::current_dir()
                .expect("Impossible de récupérer le répertoire courant")
                .to_str()
                .expect("Impossible de convertir le répertoire courant en chaîne")
        );

        // try to read the file if it exists, otherwise return an empty string
        if std::path::Path::new(&log_filepath).exists() {
            let file_content = files::read_file(&log_filepath).unwrap_or_else(|e| {
                panic!("Impossible de lire le fichier de log : {}", e);
            });
            // if the file contains already the message remove the line containing the message
            if file_content.contains(message) {
                let new_file_content = file_content
                    .lines()
                    .filter(|line| !line.contains(message))
                    .collect::<Vec<&str>>()
                    .join("\n");
                std::fs::remove_file(&log_filepath).unwrap_or_else(|e| {
                    panic!("Impossible de supprimer le fichier de log : {}", e);
                });

                files::append_to_file(&log_filepath, &new_file_content).unwrap_or_else(|e| {
                    panic!("Impossible d'écrire dans le fichier de log : {}", e);
                });
            }
        }

        // try to log the message
        utils::log(&message).log_expect(&format!("Error when logging the message : {}", &message));

        // Check if the file exists and contains the message
        let file_content = files::read_file(&log_filepath).unwrap_or_else(|e| {
            panic!("Impossible de lire le fichier de log : {}", e);
        });
        assert!(file_content.contains(message));
    }

    //%-----------------------------------------------------------------//
    //%--                                                               //
    //%-- # UNIT TEST : Remove comments                                 //
    //%--                                                               //
    //%-- Tests the remove_comments function to ensure it correctly     //
    //%-- removes                                                       //
    //%-- comments from a config file.                                  //
    //%--                                                               //
    //%-- The test reads a config file with comments, removes the       //
    //%-- comments                                                      //
    //%-- and checks if the resulting string is equal to the expected   //
    //%-- string                                                        //
    //%-- without comments.                                             //
    //%--                                                               //
    //%-- Success conditions:                                           //
    //%--     - The resulting string is equal to the expected string    //
    //%--     - The programm doesn't panic                              //
    //%--                                                               //
    //%-- Failure conditions:                                           //
    //%--     - The resulting string is not equal to the expected string//
    //%--     - The programm panics during the execution of the test    //
    //%--                                                               //
    //%-----------------------------------------------------------------//
    #[test]
    fn test_remove_comments() {
        let commented_string: String =
            String::from(include_str!("../../assets/default_config.txt"));
        let expected_string: String = String::from(include_str!(
            "../../tests/assets/default_config_no_comments.txt"
        ));

        // Remove the comments from the string
        let result = utils::remove_comments(commented_string);
        // and compare it to the expected string
        assert_eq!(result, expected_string);
    }

    //%-----------------------------------------------------------------//
    //%--                                                               //
    //%-- # UNIT TEST : Parse HashMap                                   //
    //%--                                                               //
    //%-- Tests the parse_hashmap function to ensure it correctly parses//
    //%-- a string into a HashMap.                                      //
    //%-- The test reads a config file with key-value pairs, parses the //
    //%-- string into a HashMap                                         //
    //%-- and checks if the resulting HashMap is equal to the expected  //
    //%-- HashMap.                                                      //
    //%--                                                               //
    //%-- Success conditions:                                           //
    //%--   - The resulting HashMap is equal to the expected HashMap    //
    //%--   - The programm doesn't panic                                //
    //%-- Failure conditions:                                           //
    //%--   - The resulting HashMap is not equal to the expected HashMap//
    //%--   - The programm panics during the execution of the test      //
    //%--                                                               //
    //%-----------------------------------------------------------------//
    #[test]
    fn test_parse_hashmap() {
        let input: String = String::from(include_str!("../../tests/assets/parse_hashmap.txt"));
        // Parse the input string into a HashMap
        let output: HashMap<String, String> = utils::parse_hashmap(&input, "\n", "=");

        let mut expected_output: HashMap<String, String> = HashMap::new();
        expected_output.insert(String::from("modpack_url"), String::from("baltazar"));
        expected_output.insert(String::from("modloader_url"), String::from("test1 dsds"));
        expected_output.insert(
            String::from("modloader_execname"),
            String::from("trucmuche"),
        );
        expected_output.insert(
            String::from("folders_to_overwrite"),
            String::from("mods,config,defaultconfig,kubejs,scripts,panoramas"),
        );

        // and compare it to the expected output
        assert_eq!(output, expected_output);
    }
}
