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
    fn command_exists(&self) -> Result<(), &str>
    where
        Self: Sized;
    fn remote_search(&self, query: &str) -> Result<Vec<Package>, &str>;
}
