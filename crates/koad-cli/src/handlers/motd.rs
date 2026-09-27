use anyhow::Result;
use koad_core::config::KoadConfig;
use koad_proto::citadel::v5::xp_service_client::XpServiceClient;
use koad_proto::citadel::v5::*;

pub async fn show_motd(agent_name: &str, config: &KoadConfig) -> Result<()> {
    let context = Some(crate::utils::get_trace_context(agent_name, 3));

    // 1. Fetch XP Stats
    let xp_client = XpServiceClient::connect(config.network.citadel_grpc_addr.clone())
        .await
        .ok();
    let xp_status = if let Some(mut client) = xp_client {
        client
            .get_status(XpStatusRequest {
                context: context.clone(),
                agent_name: agent_name.to_string(),
            })
            .await
            .ok()
            .map(|r| r.into_inner())
    } else {
        None
    };

    // 2. Pending inbox items (file-based agent handoff)
    let inbox =
        koad_core::inbox::pending_for(&koad_core::inbox::inbox_dir(&config.home), agent_name);

    // 3. Get Identity Info from Config
    let identity = config.identities.get(agent_name);

    // 4. Fetch Subsystem Status
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

    // Section: Stats
    println!();
    println!("\x1b[1;37m[ STATS ]\x1b[0m");
    if let Some(xp) = xp_status {
        let progress = (xp.total_xp as f32 / xp.next_level_xp as f32).min(1.0);
        let bars = (progress * 20.0) as usize;
        let bar_str = format!(
            "\x1b[32m{}\x1b[0m{}",
            "█".repeat(bars),
            "░".repeat(20 - bars)
        );

        println!(
            "  \x1b[1mTier:\x1b[0m       \x1b[32m{}\x1b[0m (Level {})",
            xp.tier_name, xp.level
        );
        println!(
            "  \x1b[1mProgress:\x1b[0m   {}  {:.1}%",
            bar_str,
            progress * 100.0
        );
        println!("  \x1b[1mTotal XP:\x1b[0m   {}", xp.total_xp);
    } else {
        println!("  \x1b[33m[OFFLINE]\x1b[0m XP service unreachable.");
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
