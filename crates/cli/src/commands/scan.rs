use libalembic::{scanner, settings::SettingsManager};

pub fn client_scan() -> anyhow::Result<()> {
    use std::io::{self, Write};

    println!("Scanning for AC client installations...");

    // Get the prefixes that will be scanned (for reporting)
    let scanned_prefixes = scanner::WineScanner::get_scannable_prefixes();

    let discovered = scanner::scan_all()?;

    let mut added_count = 0;
    let mut skipped_configs: Vec<String> = vec![];
    let mut added_configs: Vec<String> = vec![];

    // Check if there are any existing clients before we start adding
    let had_no_clients = SettingsManager::get(|s| s.clients.is_empty());

    for config in &discovered {
        // Check if already exists
        let already_exists = SettingsManager::get(|s| {
            s.clients
                .iter()
                .any(|existing| existing.install_path() == config.install_path())
        });

        if already_exists {
            skipped_configs.push(format!(
                "{} ({})",
                config.name(),
                config.install_path().display()
            ));
            continue;
        }

        // Show details and prompt
        println!();
        println!("Found: {}", config.name());
        println!("Path: {}", config.install_path().display());
        println!(
            "Type: {}",
            if config.is_wine() { "Wine" } else { "Windows" }
        );

        print!("Add this client? (y/n): ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let response = input.trim().to_lowercase();

        if response == "y" || response == "yes" {
            let should_select = had_no_clients && added_count == 0;

            SettingsManager::modify(|settings| {
                settings.add_client(config.clone(), should_select);
                settings.is_configured = true;
            })?;

            added_configs.push(format!(
                "{} ({})",
                config.name(),
                config.install_path().display()
            ));
            added_count += 1;
        } else {
            skipped_configs.push(format!(
                "{} ({})",
                config.name(),
                config.install_path().display()
            ));
        }
    }

    // Print summary report
    println!();
    println!("=== Scan Report ===");
    println!();
    println!("Scanned {} wine prefix(es):", scanned_prefixes.len());
    for prefix in &scanned_prefixes {
        println!("  - {}", prefix.display());
    }
    println!();
    println!(
        "Found {} client(s), added {}, skipped {}",
        discovered.len(),
        added_configs.len(),
        skipped_configs.len()
    );

    if !added_configs.is_empty() {
        println!();
        println!("Added:");
        for name in &added_configs {
            println!("  + {}", name);
        }
    }

    if !skipped_configs.is_empty() {
        println!();
        println!("Skipped:");
        for name in &skipped_configs {
            println!("  - {}", name);
        }
    }

    if discovered.is_empty() {
        println!();
        println!("No client installations found.");
        println!("You can add a client manually with: alembic config client add");
    }

    Ok(())
}
