mod analysis;
mod cli;
mod config;
#[cfg(target_os = "macos")]
mod daemon;
mod models;
mod server;
mod storage;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::mpsc::{channel, RecvTimeoutError};
use std::time::Duration;
use std::thread;
use tracing::{error, info};

fn get_data_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".lurk")
}

fn get_db_path() -> PathBuf {
    get_data_dir().join("events.db")
}

const SECURE_DIR_MODE: u32 = 0o700;
const SECURE_FILE_MODE: u32 = 0o600;

fn create_secure_dir(path: &PathBuf) -> Result<()> {
    if !path.exists() {
        fs::create_dir_all(path)?;
    }
    fs::set_permissions(path, Permissions::from_mode(SECURE_DIR_MODE))?;
    Ok(())
}

fn set_secure_file_permissions(path: &PathBuf) -> Result<()> {
    if path.exists() {
        fs::set_permissions(path, Permissions::from_mode(SECURE_FILE_MODE))?;
    }
    Ok(())
}

#[derive(Parser)]
#[command(name = "lurk")]
#[command(about = "A local-only keystroke logger for custom keyboard design analysis")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    #[cfg(target_os = "macos")]
    #[command(about = "Run the capture daemon (macOS only)")]
    Daemon {
        #[arg(short, long, help = "Send events to remote server (e.g., ws://server:9999)")]
        remote: Option<String>,

        #[arg(long, default_value = "true", help = "Also store events locally")]
        local: bool,

        #[arg(long, help = "Shared secret token for authenticating with remote server")]
        token: Option<String>,
    },

    #[command(about = "Run the server to receive events from remote clients")]
    Server {
        #[arg(short, long, default_value = "9999", help = "Port to listen on")]
        port: u16,

        #[arg(long, help = "Require this token from connecting daemons")]
        token: Option<String>,
    },

    #[command(about = "Export keystroke data")]
    Export {
        #[arg(short, long, default_value = "csv", help = "Output format: csv or json")]
        format: String,

        #[arg(short, long, help = "Output file path")]
        output: String,
    },

    #[command(about = "Show keystroke statistics")]
    Stats {
        #[arg(short, long, help = "Limit to last N days")]
        days: Option<u32>,
    },

    #[command(about = "Analyze typing patterns")]
    Analyze {
        #[arg(short, long, default_value = "10", help = "Number of top items to show")]
        top: usize,

        #[arg(long, default_value = "5000", help = "Max gap in ms to consider (filters outliers)")]
        max_gap: i64,

        #[arg(short, long, help = "Show detailed output including key codes and per-pair timing")]
        detailed: bool,
    },

    #[cfg(target_os = "macos")]
    #[command(about = "Check if Input Monitoring permission is granted")]
    CheckPermission,

    #[command(about = "Open interactive TUI dashboard")]
    Dashboard,

    #[command(about = "Delete old keystroke data")]
    Cleanup {
        #[arg(short, long, default_value = "90", help = "Delete events older than N days")]
        days: u32,

        #[arg(short, long, help = "Skip confirmation prompt")]
        force: bool,
    },
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("lurk=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();
    let cfg = config::load_config().unwrap_or_default();

    match cli.command {
        #[cfg(target_os = "macos")]
        None => run_daemon(cfg.daemon.remote, cfg.daemon.local.unwrap_or(true), cfg.daemon.token),
        #[cfg(target_os = "macos")]
        Some(Commands::Daemon { remote, local, token }) => {
            let effective_remote = remote.or(cfg.daemon.remote);
            let effective_token = token.or(cfg.daemon.token);
            run_daemon(effective_remote, local, effective_token)
        }
        #[cfg(not(target_os = "macos"))]
        None => {
            eprintln!("Daemon mode requires macOS. Use 'lurk server' on Linux.");
            std::process::exit(1);
        }
        Some(Commands::Server { port, token }) => {
            let effective_token = token.or(cfg.server.token);
            run_server(port, effective_token)
        }
        Some(Commands::Export { format, output }) => run_export(&format, &output),
        Some(Commands::Stats { days }) => run_stats(days),
        Some(Commands::Analyze { top, max_gap, detailed }) => run_analyze(top, max_gap, detailed),
        #[cfg(target_os = "macos")]
        Some(Commands::CheckPermission) => check_permission(),
        Some(Commands::Dashboard) => run_dashboard(),
        Some(Commands::Cleanup { days, force }) => run_cleanup(days, force),
    }
}

fn run_dashboard() -> Result<()> {
    let db_path = get_db_path();

    if !db_path.exists() {
        eprintln!("No database found at {:?}", db_path);
        eprintln!("Make sure the daemon has been run at least once.");
        return Ok(());
    }

    tui::run_dashboard(&db_path)
}

#[cfg(target_os = "macos")]
fn run_daemon(remote: Option<String>, local: bool, token: Option<String>) -> Result<()> {
    info!("Starting lurk daemon...");

    daemon::ensure_permissions()?;

    let data_dir = get_data_dir();
    create_secure_dir(&data_dir)?;

    let log_dir = data_dir.join("logs");
    create_secure_dir(&log_dir)?;

    let (tx, rx) = channel();

    match (&remote, local) {
        // Remote only - send to server, no local storage
        (Some(remote_url), false) => {
            info!("Mode: remote only");
            info!("Server: {}", remote_url);

            let config = server::RemoteClientConfig {
                url: remote_url.clone(),
                token: token.clone(),
                ..Default::default()
            };
            
            thread::spawn(move || {
                if let Err(e) = server::start_remote_client(config, rx) {
                    error!("Remote client error: {}", e);
                }
            });
        }
        
        // Remote + local - send to both (clone events)
        (Some(remote_url), true) => {
            info!("Mode: remote + local backup");
            info!("Server: {}", remote_url);

            let db_path = get_db_path();
            let db = storage::Database::new(&db_path)?;
            set_secure_file_permissions(&db_path)?;
            info!("Local database: {:?}", db_path);

            let config = server::RemoteClientConfig {
                url: remote_url.clone(),
                token: token.clone(),
                ..Default::default()
            };
            
            // Create second channel for remote
            let (remote_tx, remote_rx) = channel();
            
            // Forward events to both local DB and remote
            thread::spawn(move || {
                let mut db = db;
                let mut batch = Vec::with_capacity(100);
                let flush_interval = Duration::from_millis(50);

                loop {
                    match rx.recv_timeout(flush_interval) {
                        Ok(event) => {
                            // Forward to remote (ignore send errors, client handles reconnect)
                            let _ = remote_tx.send(event.clone());
                            batch.push(event);
                            if batch.len() >= 100 {
                                if let Err(e) = db.insert_events_batch(&batch) {
                                    error!("Failed to write batch locally: {}", e);
                                }
                                batch.clear();
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            if !batch.is_empty() {
                                if let Err(e) = db.insert_events_batch(&batch) {
                                    error!("Failed to write batch locally: {}", e);
                                }
                                batch.clear();
                            }
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            if !batch.is_empty() {
                                let _ = db.insert_events_batch(&batch);
                            }
                            break;
                        }
                    }
                }
            });
            
            // Remote client
            thread::spawn(move || {
                if let Err(e) = server::start_remote_client(config, remote_rx) {
                    error!("Remote client error: {}", e);
                }
            });
        }
        
        // Local only (default)
        (None, _) => {
            info!("Mode: local only");
            
            let db_path = get_db_path();
            let db = storage::Database::new(&db_path)?;
            set_secure_file_permissions(&db_path)?;
            info!("Database: {:?}", db_path);

            thread::spawn(move || {
                let mut db = db;
                let mut batch = Vec::with_capacity(100);
                let flush_interval = Duration::from_millis(50);

                loop {
                    match rx.recv_timeout(flush_interval) {
                        Ok(event) => {
                            batch.push(event);
                            if batch.len() >= 100 {
                                if let Err(e) = db.insert_events_batch(&batch) {
                                    error!("Failed to write batch: {}", e);
                                }
                                batch.clear();
                            }
                        }
                        Err(RecvTimeoutError::Timeout) => {
                            if !batch.is_empty() {
                                if let Err(e) = db.insert_events_batch(&batch) {
                                    error!("Failed to write batch: {}", e);
                                }
                                batch.clear();
                            }
                        }
                        Err(RecvTimeoutError::Disconnected) => {
                            if !batch.is_empty() {
                                let _ = db.insert_events_batch(&batch);
                            }
                            break;
                        }
                    }
                }
            });
        }
    }

    info!("Starting event monitor...");
    info!("Press Ctrl+C to stop");

    let monitor = daemon::EventMonitor::new(tx);
    monitor.start()?;

    Ok(())
}

fn run_server(port: u16, token: Option<String>) -> Result<()> {
    info!("Starting lurk server on port {}...", port);

    let data_dir = get_data_dir();
    create_secure_dir(&data_dir)?;

    let db_path = get_db_path();
    info!("Database: {:?}", db_path);

    // Run async server
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(server::run_server(port, &db_path, token))?;

    Ok(())
}

fn run_export(format: &str, output: &str) -> Result<()> {
    let db_path = get_db_path();

    if !db_path.exists() {
        eprintln!("No database found at {:?}", db_path);
        eprintln!("Make sure the daemon has been run at least once.");
        return Ok(());
    }

    let db = storage::Database::new(&db_path)?;

    match format {
        "csv" => cli::export_csv(&db, output)?,
        "json" => cli::export_json(&db, output)?,
        _ => {
            eprintln!("Unknown format: {}. Use 'csv' or 'json'.", format);
        }
    }

    Ok(())
}

fn run_stats(days: Option<u32>) -> Result<()> {
    let db_path = get_db_path();

    if !db_path.exists() {
        eprintln!("No database found at {:?}", db_path);
        eprintln!("Make sure the daemon has been run at least once.");
        return Ok(());
    }

    let db = storage::Database::new(&db_path)?;
    cli::show_stats(&db, days)?;

    Ok(())
}

fn run_analyze(top: usize, max_gap: i64, detailed: bool) -> Result<()> {
    let db_path = get_db_path();

    if !db_path.exists() {
        eprintln!("No database found at {:?}", db_path);
        eprintln!("Make sure the daemon has been run at least once.");
        return Ok(());
    }

    let db = storage::Database::new(&db_path)?;
    let events = db.get_all_events()?;

    if events.is_empty() {
        eprintln!("No keystroke data recorded yet.");
        return Ok(());
    }

    let filter_config = analysis::FilterConfig {
        max_gap_ms: max_gap,
        ..Default::default()
    };

    let segments = filter_config.filter_events_by_gap(&events);
    let segment_count = segments.len();
    let filtered_events: Vec<_> = segments.into_iter().flatten().cloned().collect();

    println!("=== Lurk Analysis ===\n");
    println!("Total events:     {}", events.len());
    println!("Typing segments:  {} (gaps > {}ms filtered)", segment_count, max_gap);
    println!("Analyzed events:  {}\n", filtered_events.len());

    let freq_analysis = analysis::FrequencyAnalysis::from_events(&filtered_events);

    println!("Total key presses: {}\n", freq_analysis.total_presses);

    println!("--- Top {} Keys ---", top);
    for (i, key) in freq_analysis.top_keys(top).iter().enumerate() {
        if detailed {
            println!(
                "{:2}. {:15} (0x{:02X}) {:>8} ({:.2}%)",
                i + 1,
                key.key_name,
                key.key_code,
                key.count,
                key.percentage
            );
        } else {
            println!(
                "{:2}. {:15} {:>8} ({:.2}%)",
                i + 1,
                key.key_name,
                key.count,
                key.percentage
            );
        }
    }

    println!("\n--- Top {} Bigrams ---", top);
    for (i, bigram) in freq_analysis.top_bigrams(top).iter().enumerate() {
        if detailed {
            println!(
                "{:2}. {:25} (0x{:02X}->0x{:02X}) {:>6} ({:.2}%)",
                i + 1,
                bigram.display,
                bigram.first_key,
                bigram.second_key,
                bigram.count,
                bigram.percentage
            );
        } else {
            println!(
                "{:2}. {:20} {:>8} ({:.2}%)",
                i + 1,
                bigram.display,
                bigram.count,
                bigram.percentage
            );
        }
    }

    println!("\n--- Top {} Trigrams ---", top);
    for (i, trigram) in freq_analysis.top_trigrams(top).iter().enumerate() {
        if detailed {
            println!(
                "{:2}. {:35} (0x{:02X}->0x{:02X}->0x{:02X}) {:>5} ({:.2}%)",
                i + 1,
                trigram.display,
                trigram.keys.0,
                trigram.keys.1,
                trigram.keys.2,
                trigram.count,
                trigram.percentage
            );
        } else {
            println!(
                "{:2}. {:30} {:>8} ({:.2}%)",
                i + 1,
                trigram.display,
                trigram.count,
                trigram.percentage
            );
        }
    }

    let timing = analysis::TimingAnalysis::from_events(&filtered_events, filter_config.clone());

    println!("\n--- Inter-Key Timing ---");
    println!("Samples:    {}", timing.overall_inter_key.count);
    println!("Mean:       {:.1}ms", timing.overall_inter_key.mean_ms);
    println!("Median:     {}ms", timing.overall_inter_key.median_ms);
    println!("P90:        {}ms", timing.overall_inter_key.p90_ms);
    println!("P95:        {}ms", timing.overall_inter_key.p95_ms);
    println!("P99:        {}ms", timing.overall_inter_key.p99_ms);

    if detailed && !timing.per_key_inter_key.is_empty() {
        println!("\n--- Top {} Key-Pair Timings ---", top);
        for (i, pair) in timing.top_inter_key_pairs(top).iter().enumerate() {
            println!(
                "{:2}. 0x{:02X}->0x{:02X}  mean={:.1}ms median={}ms p95={}ms (n={})",
                i + 1,
                pair.from_key,
                pair.to_key,
                pair.mean_ms,
                pair.median_ms,
                pair.p95_ms,
                pair.intervals_ms.len()
            );
        }
    }

    println!("\n--- Top {} Hold Durations ---", top);
    for (i, hold) in timing.top_hold_durations(top).iter().enumerate() {
        if detailed {
            println!(
                "{:2}. {:15} (0x{:02X}) mean={:.1}ms median={}ms p95={}ms (n={}, raw={})",
                i + 1,
                hold.key_name,
                hold.key_code,
                hold.mean_ms,
                hold.median_ms,
                hold.p95_ms,
                hold.sample_count,
                hold.durations_ms.len()
            );
        } else {
            println!(
                "{:2}. {:15} mean={:.1}ms median={}ms p95={}ms (n={})",
                i + 1,
                hold.key_name,
                hold.mean_ms,
                hold.median_ms,
                hold.p95_ms,
                hold.sample_count
            );
        }
    }

    if detailed {
        println!("\n--- Filter Config ---");
        println!("Max gap:    {}ms", timing.filter_config.max_gap_ms);
        println!("Min hold:   {}ms", timing.filter_config.min_hold_ms);
        println!("Max hold:   {}ms", timing.filter_config.max_hold_ms);
    }

    Ok(())
}

#[cfg(target_os = "macos")]
fn check_permission() -> Result<()> {
    if daemon::check_input_monitoring_permission() {
        println!("Input Monitoring permission: GRANTED");
        println!("lurk is ready to capture keystrokes.");
    } else {
        println!("Input Monitoring permission: DENIED");
        println!();
        println!("To grant permission:");
        println!("1. Open System Settings");
        println!("2. Go to Privacy & Security -> Input Monitoring");
        println!("3. Enable 'lurk'");
        println!();
        println!("Then restart the daemon.");
    }

    Ok(())
}

fn run_cleanup(days: u32, force: bool) -> Result<()> {
    use std::io::{self, Write};
    use std::time::{SystemTime, UNIX_EPOCH};

    let db_path = get_db_path();

    if !db_path.exists() {
        eprintln!("No database found at {:?}", db_path);
        return Ok(());
    }

    let db = storage::Database::new(&db_path)?;
    let total_before = db.get_total_count()?;

    if total_before == 0 {
        println!("Database is empty. Nothing to clean up.");
        return Ok(());
    }

    let cutoff_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_millis() as i64
        - (days as i64 * 24 * 60 * 60 * 1000);

    if !force {
        print!(
            "This will delete events older than {} days ({} total events in database). Continue? [y/N] ",
            days, total_before
        );
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Cleanup cancelled.");
            return Ok(());
        }
    }

    let deleted = db.cleanup_old_events(cutoff_ms)?;
    let total_after = db.get_total_count()?;

    println!("Cleanup complete:");
    println!("  Deleted: {} events", deleted);
    println!("  Remaining: {} events", total_after);

    Ok(())
}
