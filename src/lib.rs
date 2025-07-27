use std::error::Error;

pub mod pacman;

pub struct Package {
    pub repository: Vec<String>,
    pub name: String,
    pub version: String,
    pub description: Option<String>,
    pub installed: Option<bool>,
}

impl std::fmt::Display for Package {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let end = "\x1b[0m";
        let bold = "\x1b[1m";
        let green = "\x1b[32m";
        let magenta = "\x1b[35m";
        let cyan = "\x1b[36m";

        let installed = self.installed.unwrap_or(false);
        let installed_string = if installed { "[installed] " } else { "" };

        let repository_string = self.repository.join(" / ");

        let description_string = self
            .description
            .clone()
            .unwrap_or("No description provided".to_string());

        write!(
            f,
            "{bold}{} {green}{} {cyan}{}{magenta}{}{end}\n    {}",
            self.name, self.version, installed_string, repository_string, description_string
        )
    }
}

pub trait Manager {
    fn remote_search(query: &str) -> Result<Vec<Package>, Box<dyn Error>>;
}
