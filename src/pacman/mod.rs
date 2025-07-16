use crate::*;
use std::str::FromStr;
use std::{ffi::OsString, process::Command};

pub struct Pacman {
    command_exists: bool,
}

impl Manager for Pacman {
    fn command_name(&self) -> OsString {
        return OsString::from_str("pacman").unwrap();
    }
    fn test_command(&mut self) -> Result<(), String>
        where
            Self: Sized {
        if self.command_exists { return Ok(()); }
        let output = Command::new("which").arg(self.command_name()).output();
        match output {
            Ok(output) => {
                if output.status.success() {
                    self.command_exists = true;
                    return Ok(());
                } else {
                    let err = format!("{} could not be found in path", self.command_name().to_string_lossy());
                    return Err(err);
                }
            }
            Err(_) => return Err("Existence check could not be run.".to_string()),
        }
    }
    fn remote_search(&self, query: &str) -> Result<Vec<Package>, &str> {
        if self.command_exists()
        let output = Command::new
    }
}
