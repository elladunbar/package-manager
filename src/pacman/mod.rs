use crate::*;
use std::str::FromStr;
use std::{ffi::OsString, process::Command};

pub struct Pacman;

impl Manager for Pacman {
    fn command_name(&self) -> OsString {
        return OsString::from_str("pacman").unwrap();
    }
    fn command_exists(&self) -> Result<(), &str>
        where
            Self: Sized {
        let output = Command::new("which").arg("pacman").output();
        match output {
            Ok(output) => {
                if output.status.success() {
                    return Ok(());
                } else {
                    return Err("pacman could not be found in path.");
                }
            }
            Err(_) => return Err("Existence check could not be run."),
        }
    }
    fn remote_search(&self, query: &str) -> Result<Vec<Package>, &str> {
        let output = Command::new
    }
}
