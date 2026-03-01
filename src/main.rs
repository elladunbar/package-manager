use package_manager::{init_backends, list_available_backends};
use std::env;

fn print_help() {
    println!("Usage: package-manager [OPTIONS] <search-query>");
    println!();
    println!("A unified CLI for searching packages across multiple package managers.");
    println!();
    println!("Arguments:");
    println!("  <search-query>    The package name or search term");
    println!();
    println!("Options:");
    println!("  --backend=<NAME>  Specify which backend(s) to use (can be repeated)");
    println!(
        "                    Available: {}",
        list_available_backends().join(", ")
    );
    println!("  --help            Show this help message");
    println!();
    println!("Examples:");
    println!("  package-manager firefox");
    println!("  package-manager --backend=pacman firefox");
    println!("  package-manager --backend=pacman --backend=flatpak firefox");
}

fn parse_args() -> Option<(String, Vec<String>)> {
    let args: Vec<String> = env::args().collect();
    let mut query: Option<String> = None;
    let mut backends: Vec<String> = Vec::new();

    let mut i = 1;
    while i < args.len() {
        let arg = &args[i];

        if arg == "--help" {
            return None; // Signal to show help
        } else if arg.starts_with("--backend=") {
            let backend = arg.trim_start_matches("--backend=").to_string();
            backends.push(backend);
        } else if arg.starts_with('-') {
            eprintln!("Unknown option: {}", arg);
            eprintln!("Use --help for usage information");
            std::process::exit(1);
        } else if query.is_none() {
            query = Some(arg.clone());
        } else {
            eprintln!("Multiple search queries provided. Use only one.");
            eprintln!("Use --help for usage information");
            std::process::exit(1);
        }

        i += 1;
    }

    query.map(|q| (q, backends))
}

fn main() {
    init_backends();

    let result = parse_args();

    if result.is_none() {
        print_help();
        return;
    }

    let (query, requested_backends) = result.unwrap();

    // If no backend specified, default to all available backends
    let backends_to_use: Vec<String> = if requested_backends.is_empty() {
        list_available_backends()
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    } else {
        requested_backends.clone()
    };

    // Resolve backends and collect results
    let mut results: Vec<(String, Vec<package_manager::Package>)> = Vec::new();

    for backend_name in &backends_to_use {
        match package_manager::get_manager_by_name(backend_name) {
            Some(manager) => match manager.remote_search(&query) {
                Ok(packages) => {
                    if !packages.is_empty() {
                        results.push((backend_name.clone(), packages));
                    }
                }
                Err(e) => {
                    eprintln!("Error searching with '{}': {}", backend_name, e);
                }
            },
            None => {
                let available = list_available_backends();
                eprintln!(
                    "Unknown backend '{}'. Available: {}",
                    backend_name,
                    available.join(", ")
                );
                std::process::exit(1);
            }
        }
    }

    // Display grouped results
    let mut has_output = false;
    for (backend_name, packages) in &results {
        if has_output {
            println!();
        }
        println!("[{}]", backend_name.to_uppercase());
        let mut sorted_packages: Vec<_> = packages.iter().collect();
        sorted_packages.sort_by(|p1, p2| p1.name.cmp(&p2.name));
        for package in sorted_packages {
            println!("  {}", package);
        }
        has_output = true;
    }

    if !has_output {
        println!("No packages found matching '{}'", query);
    }
}
