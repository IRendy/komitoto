use clap::{Parser, Subcommand};
use std::error::Error;
use komitoto_types::{Band, Mode, Qso};
use komitoto_calc::{sunrise, geo, maidenhead};
use komitoto_service::QsoService;
use komitoto_config::ConfigFile;
use komitoto_formatter::{parse_datetime, SunriseFormatter, QsoFormatter, parse_dawn_type};
use komitoto_convert::*;

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
    /// Find CQ and ITU zone for a coordinate
    Zone {
        /// Latitude (decimal degrees, positive = North)
        #[arg(long)]
        lat: f64,

        /// Longitude (decimal degrees, positive = East)
        #[arg(long)]
        lon: f64,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Convert between Maidenhead grid locator and latitude/longitude
    Coordinate {
        /// Input format: "grid" or "latlon" (auto-detected if not specified)
        #[arg(long)]
        from: Option<String>,

        /// Output format: "grid" or "latlon" (auto-detected if not specified)
        #[arg(long)]
        to: Option<String>,

        /// Latitude (decimal degrees, positive = North)
        #[arg(long)]
        lat: Option<f64>,

        /// Longitude (decimal degrees, positive = East)
        #[arg(long)]
        lon: Option<f64>,

        /// Input value (grid string like "OL82tk" or coordinates like "39.9042,116.4074")
        #[arg(long)]
        input: Option<String>,

        /// Grid precision: 2, 4, 6, 8, or 10 (default: 6)
        #[arg(long, default_value = "6")]
        precision: usize,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Generate or decode SSTV (Slow-Scan Television) audio
    Sstv {
        #[command(subcommand)]
        action: SstvAction,
    },
}

#[derive(Subcommand)]
enum SstvAction {
    /// Encode an image to SSTV audio
    Encode {
        /// Input image file (PNG, JPEG, BMP, GIF, etc.)
        image: String,

        /// Output WAV file path (default: <image_name>.wav)
        #[arg(short, long)]
        output: Option<String>,

        /// SSTV mode (default: martinm1)
        #[arg(short, long, default_value = "martinm1")]
        mode: String,

        /// Image resize strategy: crop, fit, stretch (default: fit)
        #[arg(short, long, default_value = "fit")]
        strategy: String,
    },
    /// Decode SSTV audio to an image
    Decode {
        /// Input WAV file
        wav: String,

        /// Output image file (default: <wav_name>.png)
        #[arg(short, long)]
        output: Option<String>,

        /// SSTV mode: martinm1 or auto (default: martinm1)
        #[arg(short, long, default_value = "martinm1")]
        mode: String,
    },
    /// Show SSTV mode details
    Info {
        /// SSTV mode name
        mode: String,
    },
    /// List all supported SSTV modes
    List,
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
            let dawn_type = parse_dawn_type(dawn.as_deref())?;
            use chrono::NaiveDate;
            let times = sunrise::calc_sunrise(
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
            let meters = match geo::calc_distance(from_lat, from_lon, to_lat, to_lon) {
                Ok(m) => m,
                Err(e) => {
                    if json {
                        println!("{{");
                        println!("  \"from\": {{ \"lat\": {}, \"lon\": {} }},", from_lat, from_lon);
                        println!("  \"to\": {{ \"lat\": {}, \"lon\": {} }},", to_lat, to_lon);
                        println!("  \"error\": \"{}\"", e);
                        println!("}}");
                    } else {
                        eprintln!("Error calculating distance: {}", e);
                    }
                    return Ok(());
                }
            };
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

        CalcAction::Zone { lat, lon, json } => {
            let finder = geo::get_zone_finder();
            
            if let Some((cq_type, cq_num, itu_type, itu_num)) = finder.find_zone(lat, lon) {
                if json {
                    println!("{{");
                    println!("  \"location\": {{ \"lat\": {}, \"lon\": {} }},", lat, lon);
                    println!("  \"cq_zone\": {{ \"type\": \"{}\", \"number\": {} }},", cq_type, cq_num);
                    println!("  \"itu_zone\": {{ \"type\": \"{}\", \"number\": {} }}", itu_type, itu_num);
                    println!("}}");
                } else {
                    println!("Location: ({}, {})", lat, lon);
                    println!("CQ Zone: {}", cq_num);
                    println!("ITU Zone: {}", itu_num);
                }
            } else {
                println!("No zones found for location: ({}, {})", lat, lon);
            }
        }

        CalcAction::Coordinate { from, to: _, lat, lon, input, precision, json } => {
            
            // Determine conversion direction
            let is_grid_to_latlon = if let Some(ref from_type) = from {
                from_type.to_lowercase() == "grid"
            } else if let Some(ref input_str) = input {
                // Auto-detect: if starts with letter, it's a grid
                input_str.chars().next().map_or(false, |c| c.is_ascii_alphabetic())
            } else {
                // Default: latlon to grid
                false
            };
            
            if is_grid_to_latlon {
                // Grid → Lat/Lon
                let grid = input.ok_or("--input is required for grid to lat/lon conversion")?;
                let (lat, lon) = maidenhead::from_maidenhead(&grid)?;
                
                if json {
                    println!("{{");
                    println!("  \"input\": \"{}\",", grid);
                    println!("  \"latitude\": {},", lat);
                    println!("  \"longitude\": {}", lon);
                    println!("}}");
                } else {
                    println!("Input: {}", grid);
                    println!("Latitude: {}", lat);
                    println!("Longitude: {}", lon);
                }
            } else {
                // Lat/Lon → Grid
                let (lat, lon) = if let (Some(lat), Some(lon)) = (lat, lon) {
                    (lat, lon)
                } else if let Some(ref input_str) = input {
                    // Parse "lat,lon" format
                    let parts: Vec<&str> = input_str.split(',').collect();
                    if parts.len() != 2 {
                        return Err("Coordinates must be in 'lat,lon' format".into());
                    }
                    let lat: f64 = parts[0].trim().parse()
                        .map_err(|_| "Invalid latitude")?;
                    let lon: f64 = parts[1].trim().parse()
                        .map_err(|_| "Invalid longitude")?;
                    (lat, lon)
                } else {
                    return Err("Either --lat/--lon or --input is required for lat/lon to grid conversion".into());
                };
                
                let grid = maidenhead::to_maidenhead(lat, lon, precision)?;
                
                if json {
                    println!("{{");
                    println!("  \"latitude\": {},", lat);
                    println!("  \"longitude\": {},", lon);
                    println!("  \"grid\": \"{}\"", grid);
                    println!("}}");
                } else {
                    println!("Input: ({}, {})", lat, lon);
                    println!("Grid: {}", grid);
                }
            }
        }

        CalcAction::Sstv { action } => {
            use komitoto_sstv::{SstvEncoder, SstvDecoder, SstvMode, image_proc::ResizeStrategy};

            match action {
                SstvAction::Encode { image, output, mode, strategy } => {
                    // Validate input file exists
                    if !std::path::Path::new(&image).exists() {
                        return Err(format!("Image file not found: {}", image).into());
                    }
                    // Validate image format
                    let img_ext = std::path::Path::new(&image)
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.to_lowercase())
                        .unwrap_or_default();
                    let supported_img = ["png", "jpg", "jpeg", "bmp", "gif", "webp", "tiff", "tif"];
                    if !supported_img.contains(&img_ext.as_str()) {
                        return Err(format!(
                            "Unsupported image format: {} (supported: {})",
                            img_ext,
                            supported_img.join(", ")
                        ).into());
                    }

                    let sstv_mode = SstvMode::from_str(&mode)
                        .ok_or_else(|| format!("Unknown SSTV mode: '{}'. Use --list to see available modes.", mode))?;
                    let resize_strategy = strategy.parse::<ResizeStrategy>()
                        .map_err(|e| -> Box<dyn Error> { e.into() })?;

                    let encoder = SstvEncoder::new(sstv_mode);

                    let wav_path = output.unwrap_or_else(|| {
                        let base = std::path::Path::new(&image)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("sstv_output");
                        format!("{}.wav", base)
                    });

                    encoder.encode_to_wav(&image, &wav_path, resize_strategy)?;

                    let (w, h) = sstv_mode.resolution();
                    println!("SSTV audio generated: {}", wav_path);
                    println!("  Mode: {}", sstv_mode.name());
                    println!("  Resolution: {}x{}", w, h);
                    println!("  Image: {}", image);
                    println!("  Resize: {}", strategy);
                }
                SstvAction::Decode { wav, output, mode } => {
                    // Validate input file exists
                    if !std::path::Path::new(&wav).exists() {
                        return Err(format!("Audio file not found: {}", wav).into());
                    }
                    // Validate audio format
                    let audio_ext = std::path::Path::new(&wav)
                        .extension()
                        .and_then(|e| e.to_str())
                        .map(|e| e.to_lowercase())
                        .unwrap_or_default();
                    if !["wav", "mp3"].contains(&audio_ext.as_str()) {
                        return Err(format!(
                            "Unsupported audio format: {} (supported: wav, mp3)",
                            audio_ext
                        ).into());
                    }

                    let sstv_mode = if mode.to_lowercase() == "auto" {
                        SstvMode::MartinM1 // For now, default to M1; future: auto-detect from VIS
                    } else {
                        SstvMode::from_str(&mode)
                            .ok_or_else(|| format!("Unknown SSTV mode: '{}'. Use martinm1 or auto.", mode))?
                    };

                    let decoder = SstvDecoder::new(sstv_mode);

                    let img_path = output.unwrap_or_else(|| {
                        let base = std::path::Path::new(&wav)
                            .file_stem()
                            .and_then(|s| s.to_str())
                            .unwrap_or("decoded");
                        format!("{}.png", base)
                    });

                    decoder.decode_to_file(&wav, &img_path)?;

                    let (w, h) = sstv_mode.resolution();
                    println!("SSTV decoded: {}", img_path);
                    println!("  Mode: {}", sstv_mode.name());
                    println!("  Resolution: {}x{}", w, h);
                    println!("  Audio: {}", wav);
                }
                SstvAction::Info { mode } => {
                    let sstv_mode = SstvMode::from_str(&mode)
                        .ok_or_else(|| format!("Unknown SSTV mode: '{}'. Use --list to see available modes.", mode))?;
                    let (w, h) = sstv_mode.resolution();
                    use komitoto_sstv::spec::from_mode;
                    let spec = from_mode(sstv_mode);
                    let total_samples = spec.total_samples();
                    let duration_secs = total_samples as f64 / spec.sample_rate() as f64;
                    println!("Mode: {}", sstv_mode.name());
                    println!("Resolution: {}x{}", w, h);
                    println!("Sample rate: {} Hz", spec.sample_rate());
                    println!("Encoding: ~{:.0} seconds for a full image", duration_secs);
                }
                SstvAction::List => {
                    println!("Supported SSTV modes:");
                    for m in SstvMode::all() {
                        let (w, h) = m.resolution();
                        println!("  {} ({}x{})", m.name(), w, h);
                    }
                }
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
        let mut qsos = json_to_qsos(&json_str)
            .map_err(|e| -> Box<dyn Error> { e.into() })?;
        let mut qso = qsos.remove(0);

        // Allow CLI options to override JSON values
        if let Some(v) = call { qso.call = v.to_uppercase(); }
        if let Some(v) = freq { qso.freq = v; qso.band = Band::from_freq_mhz(v); }
        if let Some(v) = mode {
            qso.mode = Mode::from_str(&v).ok_or_else(|| format!("Unknown mode: {}", v))?;
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
        let mode_val = Mode::from_str(&mode_str)
            .ok_or_else(|| format!("Unknown mode: {}", mode_str))?;

        // Parse datetime
        let date_time_on = if now || (date.is_none() && time.is_none()) {
            chrono::Utc::now()
        } else {
            let date_str = date.ok_or("Date is required when --now is not set")?;
            parse_datetime(&date_str, time.as_deref())?
        };

        // Create QSO
        let mut qso = Qso::new(call, freq, mode_val, date_time_on);
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
        println!("{}", qsos_to_json(&qsos));
    } else {
        let output = QsoFormatter::format_qso_list(&qsos);
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
                println!("{}", qsos_to_json(&[qso.clone()]));
            } else {
                let output = QsoFormatter::format_qso_detail(&qso);
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
        qso.band = Band::from_freq_mhz(v);
    }
    if let Some(mode_str) = mode {
        qso.mode = Mode::from_str(&mode_str)
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
        println!("{}", qsos_to_json(&qsos));
    } else {
        if qsos.is_empty() {
            println!("No QSOs matching '{}' found.", call);
        } else {
            let output = QsoFormatter::format_qso_list(&qsos);
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
            adi_to_qsos(&content)
        }
        "adx" => {
            let content = std::fs::read_to_string(file)?;
            adx_to_qsos(&content)
        }
        "csv" => {
            let content = std::fs::read_to_string(file)?;
            csv_to_qsos(&content)
        }
        "json" => {
            let content = std::fs::read_to_string(file)?;
            json_to_qsos(&content)
                .map_err(|e| -> Box<dyn Error> { e.into() })?
        }
        "sqlite3" | "sqlite" | "db" => sqlite_to_qsos(file)?,
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
            let output = qsos_to_adi(&qsos);
            std::fs::write(file, &output)?;
        }
        "adx" => {
            let output = qsos_to_adx(&qsos);
            std::fs::write(file, &output)?;
        }
        "csv" => {
            let output = qsos_to_csv(&qsos);
            std::fs::write(file, &output)?;
        }
        "json" => {
            let output = qsos_to_json(&qsos);
            std::fs::write(file, &output)?;
        }
        "sqlite3" | "sqlite" | "db" => {
            qsos_to_sqlite(&qsos, file)?;
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
                        adi_to_qsos(&content)
                    }
                    "adx" => {
                        let content = std::fs::read_to_string(&file)?;
                        adx_to_qsos(&content)
                    }
                    "csv" => {
                        let content = std::fs::read_to_string(&file)?;
                        csv_to_qsos(&content)
                    }
                    "json" => {
                        let content = std::fs::read_to_string(&file)?;
                        json_to_qsos(&content)
                            .map_err(|e| -> Box<dyn Error> { e.into() })?
                    }
                    _ => return Err(format!("Unsupported format for import: {}", fmt).into()),
                };

                qsos_to_sqlite(&qsos, &temp_db)?;

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
        ConfigAction::Init => ConfigFile::init_or_exists(),
        ConfigAction::Show => ConfigFile::show(),
        ConfigAction::Set { field, value } => {
            ConfigFile::set(&field, &value)
        }
    }
}
