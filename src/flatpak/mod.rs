use crate::*;
use std::{collections::HashMap, process::Command};

struct UnmergedPackage {
    repository: String,
    name: String,
    version: String,
    description: Option<String>,
    #[allow(dead_code)]
    app_id: String, // Used for installed status matching
    installed: Option<bool>,
}

pub struct Flatpak;

impl Manager for Flatpak {
    fn name(&self) -> &'static str {
        "flatpak"
    }

    fn remote_search(&self, query: &str) -> Result<Vec<Package>, Box<dyn Error>> {
        // Get installed app IDs for checking installed status
        let installed_apps = get_installed_app_ids()?;

        let output = Command::new("flatpak").arg("search").arg(query).output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let lines: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();

        let mut unmerged_packages = Vec::new();

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // flatpak search output is tab-separated:
            // name\t| description\t| app_id\t| version\t| flavor\t| repository
            // e.g.: Firefox	Fast, Private & Safe Web Browser	org.mozilla.firefox	148.0	stable	flathub
            let parts: Vec<&str> = line.split('\t').collect();
            // flatpak search output is tab-separated:
            // name\t| description\t| app_id\t| version\t| flavor\t| repository
            // e.g.: Firefox	Fast, Private & Safe Web Browser	org.mozilla.firefox	148.0	stable	flathub
            if parts.len() < 6 {
                continue; // Skip malformed lines
            }

            let name = parts[0].to_string();
            let description = Some(parts[1].to_string());
            let app_id = parts[2].to_string();
            let version = parts[3].to_string();
            // parts[4] is flavor (stable, testing, etc.) - optional, not stored
            let repository = if parts.len() > 5 {
                parts[5].to_string()
            } else {
                "unknown".to_string()
            };

            // Check if this app is installed
            let installed = Some(installed_apps.contains(&app_id));

            let package = UnmergedPackage {
                repository,
                name,
                version,
                description,
                app_id,
                installed,
            };
            unmerged_packages.push(package);
        }

        Ok(merge_packages(unmerged_packages))
    }
}

fn get_installed_app_ids() -> Result<Vec<String>, Box<dyn Error>> {
    let output = Command::new("flatpak").arg("list").arg("--app").arg("--columns=application").output()?;
    let stdout = String::from_utf8(output.stdout)?;

    let mut app_ids = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        if !line.is_empty() {
            app_ids.push(line.to_string());
        }
    }

    Ok(app_ids)
}

fn merge_packages(unmerged: Vec<UnmergedPackage>) -> Vec<Package> {
    let mut groups: HashMap<String, Vec<UnmergedPackage>> = HashMap::new();
    for package in unmerged {
        groups
            .entry(package.name.clone())
            .or_insert_with(Vec::new)
            .push(package);
    }

    let mut result = Vec::new();
    for (name, packages) in groups {
        let repositories: Vec<String> = packages.iter().map(|p| p.repository.clone()).collect();
        let version = packages[0].version.clone();
        let description = packages[0].description.clone();
        // If any variant is installed, mark as installed
        let installed = packages.iter().find(|p| p.installed == Some(true)).and_then(|p| p.installed);
        result.push(Package {
            name,
            version,
            repository: repositories,
            description,
            installed,
        });
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_flatpak_output() {
        let line = "Firefox\tFast, Private & Safe Web Browser\torg.mozilla.firefox\t148.0\tstable\tflathub";
        let parts: Vec<&str> = line.split('\t').collect();

        assert_eq!(parts.len(), 6);
        assert_eq!(parts[0], "Firefox");
        assert_eq!(parts[1], "Fast, Private & Safe Web Browser");
        assert_eq!(parts[2], "org.mozilla.firefox");
        assert_eq!(parts[3], "148.0");
        assert_eq!(parts[4], "stable");
        assert_eq!(parts[5], "flathub");
    }

    #[test]
    fn test_merge_packages_single_package() {
        let unmerged = vec![UnmergedPackage {
            repository: "flathub".to_string(),
            name: "firefox".to_string(),
            version: "148.0".to_string(),
            description: Some("A browser".to_string()),
            app_id: "org.mozilla.firefox".to_string(),
            installed: None,
        }];

        let result = merge_packages(unmerged);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "firefox");
        assert_eq!(result[0].version, "148.0");
        assert_eq!(result[0].repository, vec!["flathub"]);
        assert_eq!(result[0].description, Some("A browser".to_string()));
        assert_eq!(result[0].installed, None);
    }

    #[test]
    fn test_merge_packages_multiple_repositories() {
        let unmerged = vec![
            UnmergedPackage {
                repository: "flathub".to_string(),
                name: "firefox".to_string(),
                version: "148.0".to_string(),
                description: Some("A browser".to_string()),
                app_id: "org.mozilla.firefox".to_string(),
                installed: None,
            },
            UnmergedPackage {
                repository: "another".to_string(),
                name: "firefox".to_string(),
                version: "147.0".to_string(),
                description: Some("A browser".to_string()),
                app_id: "org.mozilla.firefox".to_string(),
                installed: None,
            },
        ];

        let result = merge_packages(unmerged);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "firefox");
        assert_eq!(result[0].repository.len(), 2);
        assert!(result[0].repository.contains(&"flathub".to_string()));
        assert!(result[0].repository.contains(&"another".to_string()));
    }

    #[test]
    fn test_merge_packages_separate_packages() {
        let unmerged = vec![
            UnmergedPackage {
                repository: "flathub".to_string(),
                name: "firefox".to_string(),
                version: "148.0".to_string(),
                description: Some("A browser".to_string()),
                app_id: "org.mozilla.firefox".to_string(),
                installed: None,
            },
            UnmergedPackage {
                repository: "flathub".to_string(),
                name: "chrome".to_string(),
                version: "120.0".to_string(),
                description: Some("Another browser".to_string()),
                app_id: "com.google.Chrome".to_string(),
                installed: None,
            },
        ];

        let result = merge_packages(unmerged);

        assert_eq!(result.len(), 2);
        let names: Vec<&str> = result.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"firefox"));
        assert!(names.contains(&"chrome"));
    }

    #[test]
    fn test_empty_package_list() {
        let unmerged = Vec::<UnmergedPackage>::new();
        let result = merge_packages(unmerged);
        assert!(result.is_empty());
    }

    #[test]
    fn test_parse_line_minimal_fields() {
        let line = "App\tDescription\tAppID\t1.0\tstable\trepo";
        let parts: Vec<&str> = line.split('\t').collect();

        assert_eq!(parts.len(), 6);
        assert_eq!(parts[0], "App");
        assert_eq!(parts[3], "1.0");
    }

    #[test]
    fn test_parse_line_all_fields() {
        let line = "Firefox\tFast, Private & Safe Web Browser\torg.mozilla.firefox\t148.0\tstable\tflathub";
        let parts: Vec<&str> = line.split('\t').collect();

        assert_eq!(parts[0], "Firefox");
        assert_eq!(parts[1], "Fast, Private & Safe Web Browser");
        assert_eq!(parts[2], "org.mozilla.firefox");
        assert_eq!(parts[3], "148.0");
        assert_eq!(parts[4], "stable");
        assert_eq!(parts[5], "flathub");
    }
}
