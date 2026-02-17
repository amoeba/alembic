use anyhow::bail;
use libalembic::{scanner, settings::SettingsManager};

pub fn client_dll_list(client_idx: usize) -> anyhow::Result<()> {
    let dlls = SettingsManager::get(|s| s.get_client_dlls(client_idx).cloned());
    let selected_dll = SettingsManager::get(|s| {
        s.clients.get(client_idx).and_then(|c| c.selected_dll())
    });

    match dlls {
        Some(dlls) => {
            if dlls.is_empty() {
                println!("No DLLs configured for this client.");
                println!(
                    "Run 'alembic config client dll --client {} add --type <type> --path <path>'",
                    client_idx
                );
                return Ok(());
            }

            for (idx, dll) in dlls.iter().enumerate() {
                let is_selected = Some(idx) == selected_dll;
                let marker = if is_selected { " * " } else { "   " };

                println!(
                    "{}{}: {} ({})",
                    marker,
                    idx,
                    dll.dll_path.display(),
                    dll.dll_type
                );
            }

            Ok(())
        }
        None => {
            bail!(
                "Invalid client index: {}. Use 'alembic client list' to see available clients.",
                client_idx
            )
        }
    }
}

pub fn client_dll_add(
    client_idx: usize,
    dll_type: String,
    dll_path: String,
    startup_function: Option<String>,
) -> anyhow::Result<()> {
    use libalembic::inject_config::{DllType, InjectConfig};
    use std::path::PathBuf;

    // Validate client exists
    let _client =
        SettingsManager::get(|s| s.clients.get(client_idx).cloned()).ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid client index: {}. Use 'alembic client list' to see available clients.",
                client_idx
            )
        })?;

    // Parse DLL type
    let dll_type_enum = match dll_type.to_lowercase().as_str() {
        "alembic" => DllType::Alembic,
        "decal" => DllType::Decal,
        _ => {
            anyhow::bail!(
                "Invalid DLL type: {}. Must be 'alembic' or 'decal'",
                dll_type
            );
        }
    };

    // Use provided startup function or default
    let startup_fn = startup_function.or_else(|| match dll_type_enum {
        DllType::Decal => Some("DecalStartup".to_string()),
        DllType::Alembic => None,
    });

    // Create the InjectConfig
    let inject_config = InjectConfig {
        dll_type: dll_type_enum,
        dll_path: PathBuf::from(&dll_path),
        startup_function: startup_fn,
    };

    println!("Adding DLL configuration to client {}...", client_idx);
    println!("  Type: {}", inject_config.dll_type);
    println!("  Path: {}", inject_config.dll_path.display());

    SettingsManager::modify(|settings| {
        settings.add_dll_to_client(client_idx, inject_config);
    })?;

    println!();
    println!("✓ DLL configuration added!");

    Ok(())
}

pub fn client_dll_select(client_idx: usize, dll_idx: usize) -> anyhow::Result<()> {
    let dlls = SettingsManager::get(|s| s.get_client_dlls(client_idx).cloned());

    let dll_count = match dlls {
        Some(dlls) => dlls.len(),
        None => {
            bail!(
                "Invalid client index: {}. Use 'alembic client list' to see available clients.",
                client_idx
            )
        }
    };

    if dll_idx >= dll_count {
        println!("Invalid DLL index: {}", dll_idx);
        println!(
            "Use 'alembic config client dll --client {} list' to see available DLLs.",
            client_idx
        );
        return Ok(());
    }

    SettingsManager::modify(|settings| {
        settings.select_dll_for_client(client_idx, Some(dll_idx));
    })?;

    println!(
        "✓ Selected DLL at index {} for client {}",
        dll_idx, client_idx
    );

    Ok(())
}

pub fn client_dll_reset(client_idx: usize) -> anyhow::Result<()> {
    let was_selected = SettingsManager::get(|s| {
        s.clients.get(client_idx).and_then(|c| c.selected_dll())
    });

    SettingsManager::modify(|settings| {
        settings.select_dll_for_client(client_idx, None);
    })?;

    if was_selected.is_some() {
        println!("✓ DLL selection cleared for client {}", client_idx);
    } else {
        println!("No DLL was selected for client {}", client_idx);
    }

    Ok(())
}

pub fn client_dll_remove(client_idx: usize, dll_idx: usize) -> anyhow::Result<()> {
    use std::io::{self, Write};

    let dll_info = SettingsManager::get(|s| {
        s.get_client_dlls(client_idx).and_then(|dlls| {
            dlls.get(dll_idx)
                .map(|dll| (dll.dll_type.to_string(), dll.dll_path.display().to_string()))
        })
    });

    let (dll_type, dll_path) = match dll_info {
        Some(info) => info,
        None => {
            println!("Invalid DLL index: {}", dll_idx);
            println!(
                "Use 'alembic config client dll --client {} list' to see available DLLs.",
                client_idx
            );
            return Ok(());
        }
    };

    println!("This will remove the following DLL configuration:");
    println!("  [{}] {} - {}", dll_idx, dll_type, dll_path);
    println!();
    print!("Continue? (y/n): ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let response = input.trim().to_lowercase();

    if response != "y" && response != "yes" {
        println!("Cancelled.");
        return Ok(());
    }

    SettingsManager::modify(|settings| {
        settings.remove_dll_from_client(client_idx, dll_idx);
    })?;

    println!();
    println!(
        "✓ DLL configuration has been removed from client {}.",
        client_idx
    );

    Ok(())
}

pub fn client_dll_show(client_idx: usize, dll_idx: usize) -> anyhow::Result<()> {
    let dll = SettingsManager::get(|s| {
        s.get_client_dlls(client_idx)
            .and_then(|dlls| dlls.get(dll_idx).cloned())
    });

    match dll {
        Some(dll) => {
            println!(
                "DLL configuration (client {}, index {}):",
                client_idx, dll_idx
            );
            println!();
            println!("{}", dll);
        }
        None => {
            println!("Invalid client or DLL index.");
            println!(
                "Use 'alembic config client dll --client {} list' to see available DLLs.",
                client_idx
            );
        }
    }

    Ok(())
}

pub fn client_dll_edit(
    client_idx: usize,
    dll_idx: usize,
    dll_type: Option<String>,
    path: Option<String>,
    startup_function: Option<String>,
) -> anyhow::Result<()> {
    use libalembic::inject_config::DllType;
    use std::path::PathBuf;

    let dll_exists = SettingsManager::get(|s| {
        s.get_client_dlls(client_idx)
            .map(|dlls| dll_idx < dlls.len())
            .unwrap_or(false)
    });

    if !dll_exists {
        bail!(
            "Invalid client or DLL index. Use 'alembic config client dll --client {} list' to see available DLLs.",
            client_idx
        );
    }

    if dll_type.is_none() && path.is_none() && startup_function.is_none() {
        println!(
            "No changes specified. Use --type, --path, or --startup-function to modify the DLL."
        );
        return Ok(());
    }

    println!(
        "Editing DLL at index {} for client {}...",
        dll_idx, client_idx
    );

    SettingsManager::modify(|settings| {
        if let Some(dlls) = settings.get_client_dlls_mut(client_idx) {
            if let Some(dll) = dlls.get_mut(dll_idx) {
                if let Some(t) = &dll_type {
                    match t.to_lowercase().as_str() {
                        "alembic" => {
                            println!("  Updated type to: Alembic");
                            dll.dll_type = DllType::Alembic;
                        }
                        "decal" => {
                            println!("  Updated type to: Decal");
                            dll.dll_type = DllType::Decal;
                        }
                        _ => {
                            println!(
                                "  Warning: Invalid DLL type '{}', ignoring. Use 'alembic' or 'decal'.",
                                t
                            );
                        }
                    }
                }
                if let Some(p) = &path {
                    println!("  Updated path to: {}", p);
                    dll.dll_path = PathBuf::from(p);
                }
                if let Some(f) = &startup_function {
                    if f.is_empty() || f == "none" {
                        println!("  Removed startup function");
                        dll.startup_function = None;
                    } else {
                        println!("  Updated startup function to: {}", f);
                        dll.startup_function = Some(f.clone());
                    }
                }
            }
        }
    })?;

    println!("✓ DLL updated!");

    Ok(())
}

pub fn client_dll_scan(client_idx: usize) -> anyhow::Result<()> {
    use std::io::{self, Write};

    // Validate client exists
    let _client =
        SettingsManager::get(|s| s.clients.get(client_idx).cloned()).ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid client index: {}. Use 'alembic client list' to see available clients.",
                client_idx
            )
        })?;

    println!(
        "Scanning for DLL installations for client {}...",
        client_idx
    );

    let clients = SettingsManager::get(|s| s.clients.clone());

    #[cfg(not(target_os = "windows"))]
    let scanned_prefixes = scanner::get_dll_scannable_prefixes(&clients);

    let discovered_dlls = scanner::scan_for_decal_dlls(&clients)?;

    let mut added_dlls: Vec<String> = vec![];
    let mut skipped_dlls: Vec<(String, &str)> = vec![];

    for dll in &discovered_dlls {
        let already_exists = SettingsManager::get(|s| {
            s.get_client_dlls(client_idx)
                .map(|dlls| {
                    dlls.iter()
                        .any(|existing| existing.dll_path == dll.dll_path)
                })
                .unwrap_or(false)
        });

        if already_exists {
            skipped_dlls.push((format!("{} ({})", dll.dll_type, dll.dll_path.display()), "already configured"));
            continue;
        }

        println!();
        println!("Found: {}", dll.dll_path.display());
        println!("Type: {}", dll.dll_type);

        print!("Add this DLL to client {}? (y/n): ", client_idx);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let response = input.trim().to_lowercase();

        if response == "y" || response == "yes" {
            SettingsManager::modify(|settings| {
                settings.add_dll_to_client(client_idx, dll.clone());

                let current_selected =
                    settings.clients.get(client_idx).and_then(|c| c.selected_dll());

                if current_selected.is_none() && added_dlls.is_empty() {
                    settings.select_dll_for_client(client_idx, Some(0));
                }
            })?;

            added_dlls.push(format!("{} ({})", dll.dll_type, dll.dll_path.display()));
        } else {
            skipped_dlls.push((format!("{} ({})", dll.dll_type, dll.dll_path.display()), "user declined"));
        }
    }

    println!();
    println!("=== Scan Report ===");

    #[cfg(not(target_os = "windows"))]
    {
        println!();
        if scanned_prefixes.is_empty() {
            println!(
                "No clients configured. Configure a client first with: alembic config client scan"
            );
        } else {
            println!(
                "Scanned {} prefix(es) from configured clients:",
                scanned_prefixes.len()
            );
            for prefix in &scanned_prefixes {
                println!("  - {}", prefix.display());
            }
        }
    }

    println!();
    println!(
        "Found {} DLL(s), added {}, skipped {}",
        discovered_dlls.len(),
        added_dlls.len(),
        skipped_dlls.len()
    );

    if !added_dlls.is_empty() {
        println!();
        println!("Added:");
        for name in &added_dlls {
            println!("  + {}", name);
        }
    }

    if !skipped_dlls.is_empty() {
        println!();
        println!("Skipped:");
        for (name, reason) in &skipped_dlls {
            println!("  - {} ({})", name, reason);
        }
    }

    if discovered_dlls.is_empty() {
        println!();
        println!("No DLL installations found.");
        println!("Make sure Decal's Inject.dll is installed in your wine prefix.");
    }

    Ok(())
}
