use super::*;
use crate::modules::app::{AppStatus, Display};
use crate::modules::config::Config;
use crate::modules::files::{create_folder, expand_variables};
use std::env;
use std::fs::{self, File};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expand_variables() {
        let input = String::from("$HOME/magic_installer");
        let expected_output = format!(
            "{}/magic_installer",
            env::var("HOME").unwrap_or_default()
        );
        let result = expand_variables(input);
        assert_eq!(result, expected_output);
    }
}