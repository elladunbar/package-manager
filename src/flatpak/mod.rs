use crate::*;
use std::{collections::HashMap, process::Command};

struct UnmergedPackage {
    repository: String,
    name: String,
    version: String,
    description: Option<String>,
    installed: Option<bool>,
}

pub struct Flatpak;

impl Manager for Flatpak {
    fn name(&self) -> &'static str {
        "flatpak"
    }

    fn remote_search(&self, query: &str) -> Result<Vec<Package>, Box<dyn Error>> {
        let output = Command::new("flatpak").arg("search").arg(query).output()?;
        let stdout = String::from_utf8(output.stdout)?;
        let lines: Vec<String> = stdout.lines().map(|s| s.to_string()).collect();

        let mut unmerged_packages = Vec::new();

        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let name = parts[0].to_string();
            let repository = if parts.len() > 1 {
                parts[1].to_string()
            } else {
                "unknown".to_string()
            };
            let version = if parts.len() > 2 {
                parts[2].to_string()
            } else {
                "unknown".to_string()
            };
            let description = if parts.len() > 3 {
                Some(parts[3..].join(" "))
            } else {
                None
            };
            let installed = None;

            let package = UnmergedPackage {
                repository,
                name,
                version,
                description,
                installed,
            };
            unmerged_packages.push(package);
        }

        Ok(merge_packages(unmerged_packages))
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
        let repositories: Vec<String> = packages.iter().map(|p| p.repository.clone()).collect();
        let version = packages[0].version.clone();
        let description = packages[0].description.clone();
        let installed = packages[0].installed;
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
