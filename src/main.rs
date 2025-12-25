mod analysis;
mod cli;
mod daemon;
mod models;
mod storage;
mod tui;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs::{self, Permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::mpsc::channel;
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
    #[command(about = "Run the capture daemon (default)")]
    Daemon,

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

    #[command(about = "Check if Input Monitoring permission is granted")]
    CheckPermission,

    #[command(about = "Open interactive TUI dashboard")]
    Dashboard,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("lurk=info".parse().unwrap()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        None | Some(Commands::Daemon) => run_daemon(),
        Some(Commands::Export { format, output }) => run_export(&format, &output),
        Some(Commands::Stats { days }) => run_stats(days),
        Some(Commands::Analyze { top, max_gap, detailed }) => run_analyze(top, max_gap, detailed),
        Some(Commands::CheckPermission) => check_permission(),
        Some(Commands::Dashboard) => run_dashboard(),
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

fn run_daemon() -> Result<()> {
    info!("Starting lurk daemon...");

    daemon::ensure_permissions()?;

    let data_dir = get_data_dir();
    create_secure_dir(&data_dir)?;

    let log_dir = data_dir.join("logs");
    create_secure_dir(&log_dir)?;

    let db_path = get_db_path();
    let db = storage::Database::new(&db_path)?;
    set_secure_file_permissions(&db_path)?;
    info!("Database initialized: {:?}", db_path);

    let (tx, rx) = channel();

    thread::spawn(move || {
        for event in rx {
            if let Err(e) = db.insert_event(&event) {
                error!("Failed to write event: {}", e);
            }
        }
    });

    info!("Starting event monitor...");
    info!("Press Ctrl+C to stop");

    let monitor = daemon::EventMonitor::new(tx);
    monitor.start()?;

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
