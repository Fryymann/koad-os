use anyhow::Result;
use koad_core::config::KoadConfig;

pub async fn show_motd(agent_name: &str, config: &KoadConfig) -> Result<()> {
    // 1. Pending inbox items (file-based agent handoff)
    let inbox =
        koad_core::inbox::pending_for(&koad_core::inbox::inbox_dir(&config.home), agent_name);

    // 2. Get Identity Info from Config
    let identity = config.identities.get(agent_name);

    // 3. Fetch Subsystem Status
    let systems = koad_core::health::HealthRegistry::check_subsystems(config).await;

    // --- Render MOTD ---

    println!("\x1b[1;34m");
    println!("    __                      __  ____  _____");
    println!("   / /______  ____ _____  / / / /  |/  /  |");
    println!("  / //_/ __ \\/ __ `/ __ `/ / / / /|_/ / /|_|");
    println!(" / ,< / /_/ / /_/ / /_/ / /_/ / /  / / /  / ");
    println!("/_/|_|\\____/\\__,_/\\__,_/\\____/_/  /_/_/  /_/ ");
    println!("             NEURAL LINK ESTABLISHED         \x1b[0m");
    println!();

    // Section: Identity
    if let Some(id) = identity {
        println!("\x1b[1;37m[ IDENTITY ]\x1b[0m");
        println!(
            "  \x1b[1mAgent:\x1b[0m      {} (\x1b[32m{}\x1b[0m)",
            id.name, id.rank
        );
        println!("  \x1b[1mRole:\x1b[0m       {}", id.role);
        println!("  \x1b[1mBio:\x1b[0m        {}", id.bio);
    } else {
        println!("\x1b[1;37m[ IDENTITY ]\x1b[0m");
        println!("  \x1b[1mAgent:\x1b[0m      {}", agent_name);
    }

    // Section: Intelligence (Inbox/Notes)
    println!();
    println!("\x1b[1;37m[ INTELLIGENCE ]\x1b[0m");
    if inbox.is_empty() {
        println!("  Inbox empty.");
    } else {
        println!("  \x1b[1;33m{} inbox item(s):\x1b[0m", inbox.len());
        for item in inbox.iter().take(3) {
            println!("  ├─ {}", item.title);
        }
        if inbox.len() > 3 {
            println!(
                "  └─ ... (see {})",
                koad_core::inbox::inbox_dir(&config.home).display()
            );
        }
    }

    // Section: Grid Snapshot
    println!();
    println!("\x1b[1;37m[ GRID SNAPSHOT ]\x1b[0m");
    let mut grid_line = String::from("  ");
    for (i, sys) in systems.iter().enumerate() {
        let icon = match sys.status {
            koad_core::health::HealthStatus::Pass => "\x1b[32m🟢\x1b[0m",
            koad_core::health::HealthStatus::Warn => "\x1b[33m🟡\x1b[0m",
            koad_core::health::HealthStatus::Fail => "\x1b[31m🔴\x1b[0m",
            koad_core::health::HealthStatus::Unknown => "\x1b[30m⚪\x1b[0m",
        };
        grid_line.push_str(icon);
        grid_line.push(' ');
        if (i + 1) % 10 == 0 {
            println!("{}", grid_line);
            grid_line = String::from("  ");
        }
    }
    if grid_line.len() > 2 {
        println!("{}", grid_line);
    }

    println!();
    println!("\x1b[1;30m--------------------------------------------------\x1b[0m");

    Ok(())
}
