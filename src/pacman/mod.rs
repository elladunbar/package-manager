use crate::*;
use std::{collections::HashMap, process::Command};

struct UnmergedPackage {
    repository: String,
    name: String,
    version: String,
    description: Option<String>,
    installed: Option<bool>,
}

pub struct Pacman;

impl Manager for Pacman {
    fn remote_search(query: &str) -> Result<Vec<Package>, Box<dyn Error>> {
        let output = Command::new("pacman").arg("-Ss").arg(query).output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let lines: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();

        let mut unmerged_packages = Vec::new();
        let mut i = 0;
        while i + 1 < lines.len() {
            let start_line = &lines[i];
            let description_line = &lines[i + 1];

            // split into repo + name, version, and install
            let parts: Vec<&str> = start_line.split_whitespace().collect();
            let repo_name = parts[0];
            let version = parts[1].to_string();
            let installed = Some((parts.len() > 2) && parts[2].starts_with("[installed"));

            // split repo + name into repo and name
            let repo_name_parts: Vec<&str> = repo_name.split("/").collect();
            let repository = repo_name_parts[0].to_string();
            let name = repo_name_parts[1].to_string();

            // strip description
            let description = Some(description_line.trim().to_string());

            // create package
            let package = UnmergedPackage {
                repository,
                name,
                version,
                description,
                installed,
            };
            unmerged_packages.push(package);

            i += 2;
        }
        Ok(merge_packages(unmerged_packages))
    }
}

fn process_version(version: &str) -> String {
    if version.ends_with(".1") {
        version[..version.len() - 2].to_string()
    } else {
        version.to_string()
    }
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
        let processed_version = process_version(&packages[0].version);
        let repositories: Vec<String> = packages.iter().map(|p| p.repository.clone()).collect();
        let description = packages[0].description.clone();
        let installed = packages[0].installed;
        result.push(Package {
            name,
            version: processed_version,
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
    fn test_process_version_strips_trailing_1() {
        assert_eq!(process_version("1.0.1"), "1.0");
        assert_eq!(process_version("2.3.1"), "2.3");
    }

    #[test]
    fn test_process_version_keeps_normal_versions() {
        assert_eq!(process_version("1.0.0"), "1.0.0");
        assert_eq!(process_version("2.3.4"), "2.3.4");
        assert_eq!(process_version("1.0"), "1.0");
    }

    #[test]
    fn test_merge_packages_single_package() {
        let unmerged = vec![UnmergedPackage {
            repository: "core".to_string(),
            name: "firefox".to_string(),
            version: "1.0.1".to_string(),
            description: Some("A browser".to_string()),
            installed: None,
        }];

        let result = merge_packages(unmerged);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "firefox");
        assert_eq!(result[0].version, "1.0");
        assert_eq!(result[0].repository, vec!["core"]);
        assert_eq!(result[0].description, Some("A browser".to_string()));
        assert_eq!(result[0].installed, None);
    }

    #[test]
    fn test_merge_packages_multiple_repositories() {
        let unmerged = vec![
            UnmergedPackage {
                repository: "core".to_string(),
                name: "firefox".to_string(),
                version: "1.0.1".to_string(),
                description: Some("A browser".to_string()),
                installed: None,
            },
            UnmergedPackage {
                repository: "extra".to_string(),
                name: "firefox".to_string(),
                version: "1.0.1".to_string(),
                description: Some("A browser".to_string()),
                installed: None,
            },
        ];

        let result = merge_packages(unmerged);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "firefox");
        assert_eq!(result[0].version, "1.0");
        assert_eq!(result[0].repository.len(), 2);
        assert!(result[0].repository.contains(&"core".to_string()));
        assert!(result[0].repository.contains(&"extra".to_string()));
    }

    #[test]
    fn test_merge_packages_separate_different_packages() {
        let unmerged = vec![
            UnmergedPackage {
                repository: "core".to_string(),
                name: "firefox".to_string(),
                version: "1.0.1".to_string(),
                description: Some("A browser".to_string()),
                installed: None,
            },
            UnmergedPackage {
                repository: "core".to_string(),
                name: "chromium".to_string(),
                version: "2.0.1".to_string(),
                description: Some("Another browser".to_string()),
                installed: None,
            },
        ];

        let result = merge_packages(unmerged);

        assert_eq!(result.len(), 2);
        let names: Vec<&str> = result.iter().map(|p| p.name.as_str()).collect();
        assert!(names.contains(&"firefox"));
        assert!(names.contains(&"chromium"));
    }

    #[test]
    fn test_merge_packages_preserves_installed_status() {
        let unmerged = vec![
            UnmergedPackage {
                repository: "core".to_string(),
                name: "firefox".to_string(),
                version: "1.0.1".to_string(),
                description: Some("A browser".to_string()),
                installed: Some(true),
            },
            UnmergedPackage {
                repository: "extra".to_string(),
                name: "firefox".to_string(),
                version: "1.0.1".to_string(),
                description: Some("A browser".to_string()),
                installed: Some(false),
            },
        ];

        let result = merge_packages(unmerged);

        assert_eq!(result[0].installed, Some(true));
    }

    #[test]
    fn test_package_display_format() {
        let package = Package {
            repository: vec!["core".to_string(), "extra".to_string()],
            name: "firefox".to_string(),
            version: "1.0".to_string(),
            description: Some("A web browser".to_string()),
            installed: None,
        };

        let output = format!("{}", package);

        assert!(output.contains("firefox"));
        assert!(output.contains("1.0"));
        assert!(output.contains("core"));
        assert!(output.contains("extra"));
        assert!(output.contains("A web browser"));
        assert!(!output.contains("[installed]"));
    }

    #[test]
    fn test_package_display_with_installed() {
        let package = Package {
            repository: vec!["core".to_string()],
            name: "firefox".to_string(),
            version: "1.0".to_string(),
            description: Some("A web browser".to_string()),
            installed: Some(true),
        };

        let output = format!("{}", package);

        assert!(output.contains("[installed]"));
    }

    #[test]
    fn test_package_display_no_description() {
        let package = Package {
            repository: vec!["core".to_string()],
            name: "firefox".to_string(),
            version: "1.0".to_string(),
            description: None,
            installed: None,
        };

        let output = format!("{}", package);

        assert!(output.contains("No description provided"));
    }

    #[test]
    fn test_empty_package_list() {
        let unmerged = Vec::<UnmergedPackage>::new();
        let result = merge_packages(unmerged);
        assert!(result.is_empty());
    }
}
