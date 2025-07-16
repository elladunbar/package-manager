use std::ffi::OsString;

pub mod pacman;

pub struct Package {
    pub repository: Vec<String>,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
}

pub trait Manager {
    fn command_name(&self) -> OsString;
    fn test_command(&mut self) -> Result<(), String>
    where
        Self: Sized;
    fn remote_search(&self, query: &str) -> Result<Vec<Package>, &str>;
}
