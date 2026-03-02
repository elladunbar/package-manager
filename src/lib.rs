use std::error::Error;
use std::sync::OnceLock;

#[cfg(feature = "flatpak")]
pub mod flatpak;
#[cfg(feature = "homebrew")]
pub mod homebrew;
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

pub trait Manager: Send + Sync {
    fn name(&self) -> &'static str;
    fn remote_search(&self, query: &str) -> Result<Vec<Package>, Box<dyn Error>>;
}

static REGISTRY: OnceLock<Vec<&'static dyn Manager>> = OnceLock::new();

pub fn init_backends() {
    let managers: Vec<&'static dyn Manager> = vec![
        #[cfg(feature = "pacman")]
        &pacman::Pacman,
        #[cfg(feature = "flatpak")]
        &flatpak::Flatpak,
        #[cfg(feature = "homebrew")]
        &homebrew::Homebrew,
    ];
    REGISTRY.get_or_init(|| managers);
}

pub fn get_manager_by_name(name: &str) -> Option<&'static dyn Manager> {
    REGISTRY.get()?.iter().find(|m| m.name() == name).copied()
}

pub fn list_available_backends() -> Vec<&'static str> {
    REGISTRY
        .get()
        .map(|m: &Vec<&dyn Manager>| m.iter().map(|mg| mg.name()).collect())
        .unwrap_or_default()
}
