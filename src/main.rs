use clap::{Parser, Subcommand};
use std::error::Error;
use crate::types::Qso;

mod calc;
mod config;
mod config_handler;
mod convert;
mod db;
mod formatter;
mod service;
mod types;

use crate::formatter::{parse_datetime, SunriseFormatter};
use crate::service::QsoService;

#[derive(Parser)]
#[command(name = "komitoto")]
#[command(about = "HAM Radio QSO Logbook Manager")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// QSO log operations
    Log {
        #[command(subcommand)]
        action: LogAction,
    },
    /// Calculation tools
    Calc {
        #[command(subcommand)]
        action: CalcAction,
    },
    /// Manage logbooks
    Logbook {
        #[command(subcommand)]
        action: LogbookAction,
    },
    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
enum LogbookAction {
    /// Use a specific logbook file (.adi, .json, .db)
    Use {
        /// Logbook file path
        file: String,
    },
    /// List available logbooks
    List,
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Initialize configuration file
    Init,
    /// Show current configuration
    Show,
    /// Update configuration fields
    Set {
        /// Field name (callsign, name, qth, country, rig, rx, grid, timezone, my_altitude, my_antenna, my_city, tx_power, rx_power, rst_sent_default, rst_rcvd_default)
        field: String,

        /// Field value
        value: String,
    },
}

#[derive(Subcommand)]
enum LogAction {
    /// Add a new QSO record
    Add {
        /// Callsign of the station worked
        #[arg(short, long)]
        call: Option<String>,

        /// Frequency in MHz
        #[arg(short, long)]
        freq: Option<f64>,

        /// Communication mode (CW, FM, SSB, FT8, etc.)
        #[arg(short, long)]
        mode: Option<String>,

        /// QSO date in YYYYMMDD format (defaults to today)
        #[arg(long)]
        date: Option<String>,

        /// QSO time in HHMMSS format (defaults to now UTC)
        #[arg(long)]
        time: Option<String>,

        /// Use current UTC time for QSO
        #[arg(long)]
        now: bool,

        /// RST sent
        #[arg(long)]
        rst_sent: Option<String>,

        /// RST received
        #[arg(long)]
        rst_rcvd: Option<String>,

        /// Grid locator
        #[arg(long)]
        grid: Option<String>,

        /// QTH (location)
        #[arg(long)]
        qth: Option<String>,

        /// Rig used
        #[arg(long)]
        rig: Option<String>,

        /// Operator name
        #[arg(long)]
        name: Option<String>,

        /// Comment
        #[arg(long)]
        comment: Option<String>,

        /// Add QSO from JSON string
        #[arg(long)]
        json: Option<String>,
    },

    /// List QSO records
    List {
        /// Maximum number of records to show
        #[arg(short, long, default_value = "20")]
        limit: u32,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Get a specific QSO by ID
    Get {
        /// QSO ID
        id: String,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Update a QSO record
    Update {
        /// QSO ID
        id: String,

        /// Callsign
        #[arg(short, long)]
        call: Option<String>,

        /// Frequency in MHz
        #[arg(short, long)]
        freq: Option<f64>,

        /// Communication mode
        #[arg(short, long)]
        mode: Option<String>,

        /// RST sent
        #[arg(long)]
        rst_sent: Option<String>,

        /// RST received
        #[arg(long)]
        rst_rcvd: Option<String>,

        /// Grid locator
        #[arg(long)]
        grid: Option<String>,

        /// QTH (location)
        #[arg(long)]
        qth: Option<String>,

        /// Rig used
        #[arg(long)]
        rig: Option<String>,

        /// Operator name
        #[arg(long)]
        name: Option<String>,

        /// Comment
        #[arg(long)]
        comment: Option<String>,
    },

    /// Delete a QSO record
    Delete {
        /// QSO ID
        id: String,
    },

    /// Search QSO records by callsign pattern (use * as wildcard)
    Search {
        /// Callsign pattern (e.g. "BI3*", "VR2*")
        call: String,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },

    /// Import QSOs from a file (ADI, ADX, CSV, JSON, SQLite3)
    Import {
        /// Input file path
        file: String,

        /// File format: adi, adx, csv, json, sqlite3
        #[arg(short, long)]
        format: Option<String>,
    },

    /// Export QSOs to a file (ADI, ADX, CSV, JSON, SQLite3)
    Export {
        /// Output file path
        file: String,

        /// Export format: adi, adx, csv, json, sqlite3
        #[arg(short, long)]
        format: Option<String>,
    },
}

#[derive(Subcommand)]
enum CalcAction {
    /// Calculate sunrise/sunset times for a given location
    Sunrise {
        /// Latitude (decimal degrees, positive = North)
        #[arg(long)]
        lat: f64,

        /// Longitude (decimal degrees, positive = East)
        #[arg(long)]
        lon: f64,

        /// Date in YYYYMMDD format (defaults to today)
        #[arg(long)]
        date: Option<String>,

        /// Altitude in meters (default: 0)
        #[arg(long, default_value = "0.0")]
        altitude: f64,

        /// Dawn type: civil, nautical, astronomical (default: civil)
        #[arg(long)]
        dawn: Option<String>,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Calculate distance between two coordinates
    Distance {
        /// Starting latitude (decimal degrees, positive = North)
        #[arg(long)]
        from_lat: f64,

        /// Starting longitude (decimal degrees, positive = East)
        #[arg(long)]
        from_lon: f64,

        /// Ending latitude (decimal degrees, positive = North)
        #[arg(long)]
        to_lat: f64,

        /// Ending longitude (decimal degrees, positive = East)
        #[arg(long)]
        to_lon: f64,

        /// Unit: km or miles
        #[arg(long, default_value = "km")]
        unit: String,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn Error>> {
    match cli.command {
        Commands::Log { action } => run_log(action),
        Commands::Calc { action } => run_calc(action),
        Commands::Logbook { action } => run_logbook(action),
        Commands::Config { action } => run_config(action),
    }
}

/// Run logbook commands using the service layer
fn run_log(action: LogAction) -> Result<(), Box<dyn Error>> {
    let service = QsoService::new(None)?;

    match action {
        LogAction::Add {
            call,
            freq,
            mode,
            date,
            time,
            now,
            rst_sent,
            rst_rcvd,
            grid,
            qth,
            rig,
            name,
            comment,
            json,
        } => add_qso(&service, call, freq, mode, date, time, now, rst_sent, rst_rcvd,
                   grid, qth, rig, name, comment, json),

        LogAction::List { limit, json } => list_qsos(&service, limit, json),

        LogAction::Get { id, json } => get_qso(&service, id, json),

        LogAction::Update {
            id,
            call,
            freq,
            mode,
            rst_sent,
            rst_rcvd,
            grid,
            qth,
            rig,
            name,
            comment,
        } => update_qso(&service, id, call, freq, mode, rst_sent, rst_rcvd,
                       grid, qth, rig, name, comment),

        LogAction::Delete { id } => delete_qso(&service, id),

        LogAction::Search { call, json } => search_qsos(&service, call, json),

        LogAction::Import { file, format } => import_qsos(&service, &file, format.as_deref()),

        LogAction::Export { file, format } => export_qsos(&service, &file, format.as_deref()),
    }
}

/// Run calculation commands
fn run_calc(action: CalcAction) -> Result<(), Box<dyn Error>> {
    match action {
        CalcAction::Sunrise { lat, lon, date, altitude, dawn, json } => {
            let date = date.unwrap_or_else(|| chrono::Utc::now().format("%Y%m%d").to_string());
            let dawn_type = formatter::parse_dawn_type(dawn.as_deref())?;
            use chrono::NaiveDate;
            let times = calc::sunrise::calc_sunrise(
                NaiveDate::parse_from_str(&date, "%Y%m%d")?,
                lat,
                lon,
                altitude,
                dawn_type,
            )?;
            let output = SunriseFormatter::format_sunrise(&date, lat, lon, altitude, &times, json)?;
            println!("{}", output);
        }

        CalcAction::Distance { from_lat, from_lon, to_lat, to_lon, unit, json } => {
            let meters = calc::geo::calc_distance(from_lat, from_lon, to_lat, to_lon)?;
            let (value, unit_name) = match unit.to_lowercase().as_str() {
                "mile" | "miles" => (meters / 1609.344, "miles"),
                "km" | "kilometers" => (meters / 1000.0, "km"),
                _ => return Err("Unit must be 'km' or 'miles'".into()),
            };

            if json {
                println!("{{");
                println!("  \"from\": {{ \"lat\": {}, \"lon\": {} }},", from_lat, from_lon);
                println!("  \"to\": {{ \"lat\": {}, \"lon\": {} }},", to_lat, to_lon);
                println!("  \"distance_{}\": {:.2}", unit_name, value);
                println!("}}");
            } else {
                println!("Distance from ({}, {}) to ({}, {}): {:.2} {}",
                    from_lat, from_lon, to_lat, to_lon, value, unit_name);
            }
        }
    }

    Ok(())
}

/// Helper function to parse file format from extension
fn detect_format_by_ext(path: &str) -> String {
    let lower = path.to_lowercase();
    if lower.ends_with(".adi") || lower.ends_with(".adif") {
        "adi".to_string()
    } else if lower.ends_with(".adx") {
        "adx".to_string()
    } else if lower.ends_with(".csv") {
        "csv".to_string()
    } else if lower.ends_with(".json") {
        "json".to_string()
    } else if lower.ends_with(".db") || lower.ends_with(".sqlite") || lower.ends_with(".sqlite3") {
        "sqlite3".to_string()
    } else {
        "json".to_string()
    }
}

/// Add a new QSO record
fn add_qso(
    service: &QsoService,
    call: Option<String>,
    freq: Option<f64>,
    mode: Option<String>,
    date: Option<String>,
    time: Option<String>,
    now: bool,
    rst_sent: Option<String>,
    rst_rcvd: Option<String>,
    grid: Option<String>,
    qth: Option<String>,
    rig: Option<String>,
    name: Option<String>,
    comment: Option<String>,
    json: Option<String>,
) -> Result<(), Box<dyn Error>> {
    let qso = if let Some(json_str) = json {
        // Create QSO from JSON string
        let mut qsos = convert::json_to_qsos(&json_str)
            .map_err(|e| -> Box<dyn Error> { e.into() })?;
        let mut qso = qsos.remove(0);

        // Allow CLI options to override JSON values
        if let Some(v) = call { qso.call = v.to_uppercase(); }
        if let Some(v) = freq { qso.freq = v; qso.band = types::Band::from_freq_mhz(v); }
        if let Some(v) = mode {
            qso.mode = types::Mode::from_str(&v).ok_or_else(|| format!("Unknown mode: {}", v))?;
        }
        if let Some(v) = rst_sent { qso.rst_sent = Some(v); }
        if let Some(v) = rst_rcvd { qso.rst_rcvd = Some(v); }
        if let Some(v) = grid { qso.grid = Some(v); }
        if let Some(v) = qth { qso.qth = Some(v); }
        if let Some(v) = rig { qso.rig = Some(v); }
        if let Some(v) = name { qso.name = Some(v); }
        if let Some(v) = comment { qso.comment = Some(v); }

        qso
    } else {
        // Validate required fields
        let call = call.ok_or("--call is required when not using --json")?.to_uppercase();
        let freq = freq.ok_or("--freq is required when not using --json")?;
        let mode_str = mode.ok_or("--mode is required when not using --json")?;
        let mode_val = types::Mode::from_str(&mode_str)
            .ok_or_else(|| format!("Unknown mode: {}", mode_str))?;

        // Parse datetime
        let date_time_on = if now || (date.is_none() && time.is_none()) {
            chrono::Utc::now()
        } else {
            let date_str = date.ok_or("Date is required when --now is not set")?;
            parse_datetime(&date_str, time.as_deref())?
        };

        // Create QSO
        let mut qso = types::Qso::new(call, freq, mode_val, date_time_on);
        qso.rst_sent = rst_sent;
        qso.rst_rcvd = rst_rcvd;
        qso.grid = grid;
        qso.qth = qth;
        qso.rig = rig;
        qso.name = name;
        qso.comment = comment;
        qso
    };

    service.add_qso(&qso)?;
    println!("QSO added: {} ({}) @ {:.3} MHz {} {}",
        qso.call, qso.id, qso.freq, qso.mode.as_str(),
        qso.date_time_on.format("%Y-%m-%d %H:%M:%S UTC")
    );
    Ok(())
}

/// List QSO records
fn list_qsos(service: &QsoService, limit: u32, json: bool) -> Result<(), Box<dyn Error>> {
    let qsos = service.list_qsos(Some(limit))?;

    if json {
        println!("{}", convert::qsos_to_json(&qsos));
    } else {
        let output = formatter::QsoFormatter::format_qso_list(&qsos);
        println!("{}", output);
    }
    Ok(())
}

/// Get a specific QSO by ID (supports partial ID matching)
fn get_qso(service: &QsoService, id: String, json: bool) -> Result<(), Box<dyn Error>> {
    // First try exact match
    let qso_opt = service.get_qso(&id)?;

    let qso_result: Result<Qso, Box<dyn Error>> = if let Some(qso) = qso_opt {
        Ok(qso)
    } else {
        // Try prefix matching if exact match fails
        let all_qsos = service.list_qsos(None)?;
        let matches: Vec<&Qso> = all_qsos.iter()
            .filter(|qso| qso.id.starts_with(&id))
            .collect();

        match matches.len() {
            1 => Ok(matches[0].clone()),
            0 => Err(format!("QSO not found with ID '{}' or prefix '{}'", id, id).into()),
            _ => Err(format!(
                "Multiple QSOs found with prefix '{}': {} possible matches. Please use full UUID.",
                id, matches.len()
            ).into())
        }
    };

    match qso_result {
        Ok(qso) => {
            if json {
                println!("{}", convert::qsos_to_json(&[qso.clone()]));
            } else {
                let output = formatter::QsoFormatter::format_qso_detail(&qso);
                println!("{}", output);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}

/// Update a QSO record (supports partial ID matching)
fn update_qso(
    service: &QsoService,
    id: String,
    call: Option<String>,
    freq: Option<f64>,
    mode: Option<String>,
    rst_sent: Option<String>,
    rst_rcvd: Option<String>,
    grid: Option<String>,
    qth: Option<String>,
    rig: Option<String>,
    name: Option<String>,
    comment: Option<String>,
) -> Result<(), Box<dyn Error>> {
    // Try exact match first
    let mut qso = match service.get_qso(&id)? {
        Some(qso) => qso,
        None => {
            // Try prefix matching
            let all_qsos = service.list_qsos(None)?;
            let matches: Vec<&Qso> = all_qsos.iter()
                .filter(|qso| qso.id.starts_with(&id))
                .collect();

            match matches.len() {
                0 => return Err(format!("QSO not found with ID '{}' or prefix '{}'", id, id).into()),
                1 => matches[0].clone(),
                _ => return Err(format!(
                    "Multiple QSOs found with prefix '{}': {} possible matches. Please use full UUID.",
                    id, matches.len()
                ).into()),
            }
        }
    };

    if let Some(v) = call { qso.call = v; }
    if let Some(v) = freq {
        qso.freq = v;
        qso.band = types::Band::from_freq_mhz(v);
    }
    if let Some(mode_str) = mode {
        qso.mode = types::Mode::from_str(&mode_str)
            .ok_or_else(|| format!("Unknown mode: {}", mode_str))?;
    }
    if let Some(v) = rst_sent { qso.rst_sent = Some(v); }
    if let Some(v) = rst_rcvd { qso.rst_rcvd = Some(v); }
    if let Some(v) = grid { qso.grid = Some(v); }
    if let Some(v) = qth { qso.qth = Some(v); }
    if let Some(v) = rig { qso.rig = Some(v); }
    if let Some(v) = name { qso.name = Some(v); }
    if let Some(v) = comment { qso.comment = Some(v); }

    service.update_qso(&qso)?;
    println!("QSO updated: {}", qso.id);
    Ok(())
}

/// Delete a QSO record (supports partial ID matching)
fn delete_qso(service: &QsoService, id: String) -> Result<(), Box<dyn Error>> {
    // Try exact match first
    let deleted = service.delete_qso(&id)?;

    if deleted {
        println!("QSO deleted: {}", id);
    } else {
        // Try prefix matching
        let all_qsos = service.list_qsos(None)?;
        let matches: Vec<&Qso> = all_qsos.iter()
            .filter(|qso| qso.id.starts_with(&id))
            .collect();

        match matches.len() {
            0 => eprintln!("QSO not found with ID '{}' or prefix '{}'", id, id),
            1 => {
                if service.delete_qso(&matches[0].id).unwrap_or(false) {
                    println!("QSO deleted (matched by prefix): {} -> {}", id, matches[0].id);
                } else {
                    eprintln!("Failed to delete QSO");
                }
            }
            _ => eprintln!(
                "Multiple QSOs found with prefix '{}': {} possible matches. Please use full UUID.",
                id, matches.len()
            ),
        }
    }
    Ok(())
}

/// Search QSO records by callsign pattern
fn search_qsos(service: &QsoService, call: String, json: bool) -> Result<(), Box<dyn Error>> {
    let qsos = service.search_qsos(&call)?;

    if json {
        println!("{}", convert::qsos_to_json(&qsos));
    } else {
        if qsos.is_empty() {
            println!("No QSOs matching '{}' found.", call);
        } else {
            let output = formatter::QsoFormatter::format_qso_list(&qsos);
            println!("{}", output);
        }
    }
    Ok(())
}

/// Import QSOs from a file
fn import_qsos(service: &QsoService, file: &str, format: Option<&str>) -> Result<(), Box<dyn Error>> {
    let fmt = match format {
        Some(f) => f.to_string(),
        None => detect_format_by_ext(file),
    };

    let qsos = match fmt.as_str() {
        "adi" | "adif" => {
            let content = std::fs::read_to_string(file)?;
            convert::adi_to_qsos(&content)
        }
        "adx" => {
            let content = std::fs::read_to_string(file)?;
            convert::adx_to_qsos(&content)
        }
        "csv" => {
            let content = std::fs::read_to_string(file)?;
            convert::csv_to_qsos(&content)
        }
        "json" => {
            let content = std::fs::read_to_string(file)?;
            convert::json_to_qsos(&content)
                .map_err(|e| -> Box<dyn Error> { e.into() })?
        }
        "sqlite3" | "sqlite" | "db" => convert::sqlite_to_qsos(file)?,
        _ => return Err(format!("Unknown format: {}", fmt).into()),
    };

    for qso in &qsos {
        service.add_qso(qso)?;
    }
    println!("Imported {} QSO(s) from {} (format: {})", qsos.len(), file, fmt);
    Ok(())
}

/// Export QSOs to a file
fn export_qsos(service: &QsoService, file: &str, format: Option<&str>) -> Result<(), Box<dyn Error>> {
    let qsos = service.get_all_qsos()?;
    let fmt = match format {
        Some(f) => f.to_string(),
        None => detect_format_by_ext(file),
    };

    match fmt.as_str() {
        "adi" | "adif" => {
            let output = convert::qsos_to_adi(&qsos);
            std::fs::write(file, &output)?;
        }
        "adx" => {
            let output = convert::qsos_to_adx(&qsos);
            std::fs::write(file, &output)?;
        }
        "csv" => {
            let output = convert::qsos_to_csv(&qsos);
            std::fs::write(file, &output)?;
        }
        "json" => {
            let output = convert::qsos_to_json(&qsos);
            std::fs::write(file, &output)?;
        }
        "sqlite3" | "sqlite" | "db" => {
            convert::qsos_to_sqlite(&qsos, file)?;
        }
        _ => return Err(format!("Unknown format: {}", fmt).into()),
    };

    println!("Exported {} QSO(s) to {} (format: {})", qsos.len(), file, fmt);
    Ok(())
}

/// Run logbook management commands
fn run_logbook(action: LogbookAction) -> Result<(), Box<dyn Error>> {
    match action {
        LogbookAction::Use { file } => {
            // Check if it's a database file
            let logbook_to_use: std::path::PathBuf;

            if file.ends_with(".db") || file.ends_with(".sqlite") || file.ends_with(".sqlite3") {
                if !std::path::Path::new(&file).exists() {
                    return Err(format!("File not found: {}", file).into());
                }
                logbook_to_use = std::path::PathBuf::from(file.clone());
                println!("Using logbook: {}", file);
            } else {
                // For non-database files, create a temporary database
                let base_name: String = std::path::Path::new(&file)
                    .with_extension("")
                    .to_string_lossy()
                    .into_owned();
                let temp_db: String = format!("{}_temp.db", base_name);

                println!("Importing {} to temporary database...", file);

                let fmt = detect_format_by_ext(&file);
                let qsos = match fmt.as_str() {
                    "adi" | "adif" => {
                        let content = std::fs::read_to_string(&file)?;
                        convert::adi_to_qsos(&content)
                    }
                    "adx" => {
                        let content = std::fs::read_to_string(&file)?;
                        convert::adx_to_qsos(&content)
                    }
                    "csv" => {
                        let content = std::fs::read_to_string(&file)?;
                        convert::csv_to_qsos(&content)
                    }
                    "json" => {
                        let content = std::fs::read_to_string(&file)?;
                        convert::json_to_qsos(&content)
                            .map_err(|e| -> Box<dyn Error> { e.into() })?
                    }
                    _ => return Err(format!("Unsupported format for import: {}", fmt).into()),
                };

                convert::qsos_to_sqlite(&qsos, &temp_db)?;

                println!("Created temporary database: {} with {} QSOs", temp_db, qsos.len());
                logbook_to_use = std::path::PathBuf::from(temp_db);
            }

            unsafe {
                std::env::set_var("KOMITOTO_LOGBOOK", logbook_to_use.to_str().unwrap_or(""));
            }
            Ok(())
        }

        LogbookAction::List => {
            println!("Available logbooks in current directory:");

            let entries = std::fs::read_dir(".")?;
            let mut files: Vec<_> = entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let path = e.path();
                    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
                    matches!(ext, "adi" | "adx" | "json" | "csv" | "db" | "sqlite" | "sqlite3")
                })
                .map(|e| e.path())
                .collect();

            files.sort();

            // Fix ownership issue - use references instead of moving
            for path in &files {
                if path.file_name().map_or(false, |n| n != "komitoto.db") {
                    println!("  - {}", path.display());
                }
            }

            println!("  - komitoto.db (default)");
            if files.is_empty() {
                println!("\nNo additional logbooks found. Run 'komitoto log import <file>' to add one.");
            }

            Ok(())
        }
    }
}

/// Run configuration commands
fn run_config(action: ConfigAction) -> Result<(), Box<dyn Error>> {
    match action {
        ConfigAction::Init => config_handler::ConfigFile::init_or_exists(),
        ConfigAction::Show => config_handler::ConfigFile::show(),
        ConfigAction::Set { field, value } => {
            config_handler::ConfigFile::set(&field, &value)
        }
    }
}
