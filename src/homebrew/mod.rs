use crate::*;
use serde::Deserialize;
use std::{collections::HashMap, process::Command};

#[derive(Deserialize)]
struct BrewInfoJson {
    formulae: Vec<Formula>,
    casks: Vec<Cask>,
}

#[derive(Deserialize)]
struct Formula {
    name: String,
    tap: String,
    desc: Option<String>,
    versions: Versions,
    installed: Option<Vec<InstalledInfo>>,
}

#[derive(Deserialize)]
struct Versions {
    stable: Option<String>,
    head: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct InstalledInfo {
    version: String,
    installed_as_dependency: bool,
    installed_on_request: bool,
}

#[derive(Deserialize)]
struct Cask {
    token: String,
    tap: String,
    desc: Option<String>,
    version: String,
    installed: Option<String>,
}

struct UnmergedPackage {
    repository: String,
    name: String,
    version: String,
    description: Option<String>,
    installed: Option<bool>,
}

pub struct Homebrew;

impl Manager for Homebrew {
    fn name(&self) -> &'static str {
        "homebrew"
    }

    fn remote_search(&self, query: &str) -> Result<Vec<Package>, Box<dyn std::error::Error>> {
        let search_output = Command::new("brew")
            .arg("search")
            .arg(format!("/{}/", query))
            .output()?;

        let stdout = String::from_utf8(search_output.stdout)?;
        let package_names = parse_search_output(&stdout);

        let mut unmerged_packages = Vec::new();

        for package_name in package_names {
            let info_output = Command::new("brew")
                .arg("info")
                .arg("--json=v2")
                .arg(&package_name)
                .output()?;

            let stdout = String::from_utf8(info_output.stdout)?;
            let json: BrewInfoJson = serde_json::from_str(&stdout)?;

            for formula in json.formulae {
                let stable_version = get_version_from_versions(&formula.versions);
                let installed = formula
                    .installed
                    .as_ref()
                    .filter(|i| !i.is_empty())
                    .and_then(|i| i.first())
                    .map(|i| i.version == stable_version);
                unmerged_packages.push(UnmergedPackage {
                    repository: formula.tap,
                    name: formula.name,
                    version: stable_version,
                    description: formula.desc,
                    installed: installed.map(|_| true),
                });
            }

            for cask in json.casks {
                unmerged_packages.push(UnmergedPackage {
                    repository: cask.tap,
                    name: cask.token,
                    version: cask.version,
                    description: cask.desc,
                    installed: Some(cask.installed.is_some()),
                });
            }
        }

        Ok(merge_packages(unmerged_packages))
    }
}

fn parse_search_output(output: &str) -> Vec<String> {
    let mut names = Vec::new();

    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        names.push(parts[0].to_string());
    }

    names
}

fn get_version_from_versions(versions: &Versions) -> String {
    versions
        .stable
        .clone()
        .or_else(|| versions.head.clone())
        .unwrap_or_else(|| "unknown".to_string())
}

fn merge_packages(unmerged: Vec<UnmergedPackage>) -> Vec<Package> {
    let mut groups: HashMap<String, Vec<UnmergedPackage>> = HashMap::new();
    for package in unmerged {
        groups
            .entry(package.name.clone())
            .or_default()
            .push(package);
    }

    let mut result = Vec::new();
    for (name, packages) in groups {
        let repositories: Vec<String> = packages.iter().map(|p| p.repository.clone()).collect();

        let (version, description, installed) =
            if let Some(installed_pkg) = packages.iter().find(|p| p.installed == Some(true)) {
                (
                    installed_pkg.version.clone(),
                    installed_pkg.description.clone(),
                    Some(true),
                )
            } else if let Some(max_pkg) = find_newest_version(&packages) {
                (
                    max_pkg.version.clone(),
                    max_pkg.description.clone(),
                    Some(false),
                )
            } else {
                let pkg = &packages[0];
                (pkg.version.clone(), pkg.description.clone(), pkg.installed)
            };

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

fn find_newest_version(packages: &[UnmergedPackage]) -> Option<&UnmergedPackage> {
    let mut newest: Option<&UnmergedPackage> = None;
    for pkg in packages {
        if let Some(current) = newest {
            if parse_version(&pkg.version) > parse_version(&current.version) {
                newest = Some(pkg);
            }
        } else {
            newest = Some(pkg);
        }
    }
    newest
}

fn parse_version(version: &str) -> Vec<u32> {
    version
        .trim()
        .split('.')
        .filter_map(|s| s.parse::<u32>().ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search_output_with_tap() {
        let output = "anomalyco/tap/opencode\tonline IDE and package manager";
        let names = parse_search_output(output);
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "anomalyco/tap/opencode");
    }

    #[test]
    fn test_parse_search_output_without_tap() {
        let output = "firefox\tWeb browser";
        let names = parse_search_output(output);
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "firefox");
    }

    #[test]
    fn test_parse_search_output_empty_lines() {
        let output = "\n\nfirefox\tWeb browser\n\n";
        let names = parse_search_output(output);
        assert_eq!(names.len(), 1);
        assert_eq!(names[0], "firefox");
    }

    #[test]
    fn test_get_version_from_versions_stable() {
        let versions = Versions {
            stable: Some("1.0.0".to_string()),
            head: Some("HEAD".to_string()),
        };
        assert_eq!(get_version_from_versions(&versions), "1.0.0");
    }

    #[test]
    fn test_get_version_from_versions_no_stable() {
        let versions = Versions {
            stable: None,
            head: Some("HEAD".to_string()),
        };
        assert_eq!(get_version_from_versions(&versions), "HEAD");
    }

    #[test]
    fn test_get_version_from_versions_empty() {
        let versions = Versions {
            stable: None,
            head: None,
        };
        assert_eq!(get_version_from_versions(&versions), "unknown");
    }

    #[test]
    fn test_merge_packages_single_package() {
        let unmerged = vec![UnmergedPackage {
            repository: "homebrew/core".to_string(),
            name: "firefox".to_string(),
            version: "1.0.0".to_string(),
            description: Some("A browser".to_string()),
            installed: None,
        }];

        let result = merge_packages(unmerged);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "firefox");
        assert_eq!(result[0].version, "1.0.0");
        assert_eq!(result[0].repository, vec!["homebrew/core"]);
    }

    #[test]
    fn test_merge_packages_separate_different_repos_same_name() {
        let unmerged = vec![
            UnmergedPackage {
                repository: "homebrew/core".to_string(),
                name: "mpv".to_string(),
                version: "1.0".to_string(),
                description: Some("Media player from core".to_string()),
                installed: None,
            },
            UnmergedPackage {
                repository: "anomalyco/tap".to_string(),
                name: "mpv".to_string(),
                version: "2.0".to_string(),
                description: Some("Media player from custom tap".to_string()),
                installed: None,
            },
        ];

        let result = merge_packages(unmerged);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].name, "mpv");
        assert_eq!(result[0].version, "2.0");
        assert_eq!(result[0].repository.len(), 2);
    }

    #[test]
    fn test_merge_packages_installed_takes_precedence() {
        let unmerged = vec![
            UnmergedPackage {
                repository: "homebrew/core".to_string(),
                name: "mpv".to_string(),
                version: "1.0".to_string(),
                description: Some("Core version".to_string()),
                installed: Some(false),
            },
            UnmergedPackage {
                repository: "anomalyco/tap".to_string(),
                name: "mpv".to_string(),
                version: "2.0".to_string(),
                description: Some("Custom tap version".to_string()),
                installed: Some(true),
            },
        ];

        let result = merge_packages(unmerged);

        assert_eq!(result.len(), 1);
        assert_eq!(result[0].version, "2.0");
        assert_eq!(
            result[0].description,
            Some("Custom tap version".to_string())
        );
        assert_eq!(result[0].installed, Some(true));
    }

    #[test]
    fn test_parse_version() {
        assert_eq!(parse_version("1.0"), vec![1, 0]);
        assert_eq!(parse_version("1.2.15"), vec![1, 2, 15]);
        assert_eq!(parse_version("1.2.10"), vec![1, 2, 10]);
        assert!(parse_version("2.0") > parse_version("1.9.9"));
    }
}
