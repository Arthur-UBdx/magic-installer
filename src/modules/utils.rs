#[allow(unused_imports)]
use crate::modules::{app, config, files};

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- ## SP-PT-001                                                  //
//%--                                                               //
//%-- L'utilitaire doit être capable de fonctionner sur un système  //
//%-- d'exploitation Windows                                        //
//%-- et Linux.                                                     //
//%--                                                               //
//%-----------------------------------------------------------------//
const WIN_VARIABLES_REGEX: &str = r"%([A-z]+)%";
const LINUX_VARIABLES_REGEX: &str = r"\$([A-z]+)";

/// This trait is used to unwrap a Result and log the error if it occurs.
pub trait UnwrapOrLog<T> {
    fn unwrap_or_log_panic(self, message: &str) -> T;
}

/// It is a custom implementation of the `unwrap_or` method for Result types.
/// It takes a Result<T, E> and returns T if the result is Ok.
/// If the result is an Err, it logs the error message and panics.
/// This is useful for handling errors in a consistent way throughout the application.
impl<T, E: std::fmt::Debug> UnwrapOrLog<T> for Result<T, E> {
    fn unwrap_or_log_panic(self, message: &str) -> T {
        match self {
            Ok(value) => value,
            Err(error) => {
                panic_log(format!("Error: {:?}\n\nMessage: {}", error, message));
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
/// It is similar to the `UnwrapOrLog` trait for Result types.
impl<T> UnwrapOrLog<T> for Option<T> {
    fn unwrap_or_log_panic(self, message: &str) -> T {
        match self {
            Some(value) => value,
            _ => {
                panic_log(String::from(format!(
                    "Error: None value unwrapped\nmessage: {}",
                    message
                )));
                // panic!();
            }
        }
    }
}

/// Log a message to the debug file.
pub fn log(message: &str) -> () {
    files::append_to_file(
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

//%-----------------------------------------------------------------//
//%--                                                               //
//%-- # UNIT TESTS:                                                 //
//%--                                                               //
//%-----------------------------------------------------------------//

#[cfg(test)]
mod tests {
    use crate::modules::{config, files, utils};
    use std::env;

    #[test]
    fn test_expand_variables() {
        // Set the HOME environment variable explicitly for the test
        // The tests have to be run for each OS type
        env::set_var("HOME", "/test/home");
        let input: String = match config::OS_TYPE {
            config::OSType::Windows => String::from("%HOME%/magic_installer"),
            config::OSType::Linux => String::from("$HOME/magic_installer"),
        };

        let expected_output = String::from("/test/home/magic_installer");
        let result = utils::expand_variables(input);
        assert_eq!(result, expected_output);
    }

    #[test]
    fn test_log() {
        let message = "This is a test log message\n";

        let filepath = match config::OS_TYPE {
            config::OSType::Windows => {
                format!(
                    "{}{}",
                    env::var("Temp").unwrap_or_else(|_| {
                        panic!("Impossible de récupérer la variable d'environnement Temp")
                    }),
                    "test_log.txt"
                )
            }
            config::OSType::Linux => "/tmp/test_log.txt".to_string(),
        };

        files::append_to_file(&filepath, message).unwrap_or_else(|_| {
            panic!("Impossible d'écrire dans le fichier de log : {}", &filepath);
        });

        // Check if the file exists and contains the message
        let file_content = files::read_file(&filepath).unwrap_or_else(|_| {
            panic!("Impossible de lire le fichier de log : {}", &filepath);
        });
        assert!(file_content.contains(message));
        // Clean up the test log file
        std::fs::remove_file(&filepath).unwrap_or_else(|_| {
            panic!("Impossible de supprimer le fichier de log : {}", &filepath);
        });
    }
}
