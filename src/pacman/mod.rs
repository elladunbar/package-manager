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
