use clap::{Parser, Subcommand};
use std::error::Error;
use komitoto_types::{Band, Mode, Qso};
use komitoto_calc::{sunrise, geo, maidenhead};
use komitoto_service::QsoService;
use komitoto_config::ConfigFile;
use komitoto_formatter::{parse_datetime, SunriseFormatter, QsoFormatter, parse_dawn_type};
use komitoto_convert::*;
use komitoto_ssdv;

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
    /// Utility tools
    Tools {
        #[command(subcommand)]
        action: ToolsAction,
    },
    /// Generate or decode SSTV (Slow-Scan Television) audio
    Sstv {
        #[command(subcommand)]
        action: SstvAction,
    },
    /// Encode or decode SSDV (Slow Scan Digital Video) packets
    Ssdv {
        #[command(subcommand)]
        action: SsdvAction,
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
enum ToolsAction {
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
    /// Calculate wavelength from frequency
    Wavelength {
        /// Frequency in MHz
        #[arg(long)]
        freq: f64,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Calculate frequency from wavelength
    Frequency {
        /// Wavelength in meters
        #[arg(long)]
        wavelength: f64,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
    },
    /// Validate and display callsign information
    Callsign {
        /// Callsign to validate (e.g. BD7ACE, W1AW, JA1AA)
        callsign: String,

        /// Output as JSON
        #[arg(short, long)]
        json: bool,
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

#[derive(Subcommand)]
enum SsdvAction {
    /// Encode a JPEG image to SSDV packets
    Encode {
        /// Input JPEG image file
        image: String,

        /// Output binary file (default: <input>.bin)
        #[arg(short, long)]
        output: Option<String>,

        /// Callsign (max 6 chars, A-Z 0-9, required)
        #[arg(short, long)]
        callsign: String,

        /// Image ID (0-255, default: 0)
        #[arg(short, long, default_value = "0")]
        id: u8,

        /// Quality level 0-7 (default: 4)
        #[arg(short, long, default_value = "4")]
        quality: i8,

        /// Disable forward error correction (NOFEC mode)
        #[arg(short, long)]
        no_fec: bool,

        /// Packet size, max 256 (default: 256)
        #[arg(short, long, default_value = "256")]
        pkt_size: usize,
    },
    /// Decode SSDV packets to a JPEG image
    Decode {
        /// Input SSDV binary file
        input: String,

        /// Output JPEG file (default: <input>.jpeg)
        #[arg(short, long)]
        output: Option<String>,

        /// Print packet details during decoding
        #[arg(short, long)]
        verbose: bool,

        /// Drop test: randomly discard N% of packets (0-100)
        #[arg(short = 't', long)]
        drop: Option<u8>,

        /// Packet size (default: 256)
        #[arg(short, long, default_value = "256")]
        pkt_size: usize,
    },
    /// Show SSDV packet header information
    Info {
        /// Input SSDV binary file
        input: String,

        /// Packet size (default: 256)
        #[arg(short, long, default_value = "256")]
        pkt_size: usize,
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
        Commands::Tools { action } => run_tools(action),
        Commands::Sstv { action } => run_sstv(action),
        Commands::Ssdv { action } => run_ssdv(action),
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

/// Run utility tools commands
fn run_tools(action: ToolsAction) -> Result<(), Box<dyn Error>> {
    match action {
        ToolsAction::Sunrise { lat, lon, date, altitude, dawn, json } => {
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

        ToolsAction::Distance { from_lat, from_lon, to_lat, to_lon, unit, json } => {
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

        ToolsAction::Zone { lat, lon, json } => {
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

        ToolsAction::Coordinate { from, to: _, lat, lon, input, precision, json } => {
            
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

        ToolsAction::Wavelength { freq, json } => {
            if freq <= 0.0 {
                return Err("Frequency must be positive".into());
            }
            if freq.is_infinite() || freq.is_nan() {
                return Err("Frequency must be a finite number".into());
            }
            const C: f64 = 299_792_458.0; // speed of light in m/s
            let freq_hz = freq * 1_000_000.0;
            let wavelength = C / freq_hz;
            let band = Band::from_freq_mhz(freq);

            if json {
                println!("{{");
                println!("  \"frequency_mhz\": {},", freq);
                println!("  \"wavelength_m\": {:.6},", wavelength);
                if let Some(b) = band {
                    println!("  \"band\": \"{}\"", b.name());
                } else {
                    println!("  \"band\": null");
                }
                println!("}}");
            } else {
                println!("Frequency: {} MHz", freq);
                println!("Wavelength: {:.6} m", wavelength);
                if let Some(b) = band {
                    println!("Band: {}", b.name());
                }
            }
        }

        ToolsAction::Frequency { wavelength, json } => {
            if wavelength <= 0.0 {
                return Err("Wavelength must be positive".into());
            }
            if wavelength.is_infinite() || wavelength.is_nan() {
                return Err("Wavelength must be a finite number".into());
            }
            const C: f64 = 299_792_458.0; // speed of light in m/s
            let freq_hz = C / wavelength;
            let freq_mhz = freq_hz / 1_000_000.0;
            let band = Band::from_freq_mhz(freq_mhz);

            if json {
                println!("{{");
                println!("  \"wavelength_m\": {},", wavelength);
                println!("  \"frequency_mhz\": {:.6},", freq_mhz);
                if let Some(b) = band {
                    println!("  \"band\": \"{}\"", b.name());
                } else {
                    println!("  \"band\": null");
                }
                println!("}}");
            } else {
                println!("Wavelength: {} m", wavelength);
                println!("Frequency: {:.6} MHz", freq_mhz);
                if let Some(b) = band {
                    println!("Band: {}", b.name());
                }
            }
        }

        ToolsAction::Callsign { callsign, json } => {
            use komitoto_ssdv::{encode_callsign, is_valid_ham_callsign, itu_prefix_info, validate_callsign};

            let cs = callsign.to_uppercase();
            let valid_ssdv = validate_callsign(&cs);
            let valid_ham = is_valid_ham_callsign(&cs);
            let encoded = encode_callsign(&cs);
            let country = itu_prefix_info(&cs);

            if json {
                println!("{{");
                println!("  \"callsign\": \"{}\",", cs);
                println!("  \"valid_ssdv\": {},", valid_ssdv.is_ok());
                println!("  \"valid_ham\": {},", valid_ham);
                println!("  \"base40\": {},", encoded);
                println!("  \"itu_country\": \"{}\"", country);
                println!("}}");
            } else {
                println!("Callsign: {}", cs);
                if valid_ssdv.is_ok() {
                    println!("  SSDV: valid");
                } else {
                    println!("  SSDV: invalid ({})", valid_ssdv.unwrap_err());
                }
                if valid_ham {
                    println!("  ITU format: valid");
                } else {
                    println!("  ITU format: invalid");
                }
                println!("  Base-40: {}", encoded);
                println!("  ITU country: {}", country);
            }
        }
    }

    Ok(())
}

/// Run SSTV commands
fn run_sstv(action: SstvAction) -> Result<(), Box<dyn Error>> {
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

    Ok(())
}

/// Run SSDV commands
fn run_ssdv(action: SsdvAction) -> Result<(), Box<dyn Error>> {
    use komitoto_ssdv::{SsdvEncoder, SsdvDecoder, PacketType};

    match action {
        SsdvAction::Encode { image, output, callsign, id, quality, no_fec, pkt_size } => {
            if !std::path::Path::new(&image).exists() {
                return Err(format!("Image file not found: {}", image).into());
            }

            let raw_data = std::fs::read(&image)?;

            // If already JPEG, use directly; otherwise convert via image crate
            let jpeg_data = if raw_data.len() >= 2 && raw_data[0] == 0xFF && raw_data[1] == 0xD8 {
                raw_data
            } else {
                let img = image::open(&image)
                    .map_err(|e| format!("Failed to open image '{}': {}", image, e))?;
                let mut buf = std::io::Cursor::new(Vec::new());
                img.write_to(&mut buf, image::ImageFormat::Jpeg)
                    .map_err(|e| format!("Failed to convert to JPEG: {}", e))?;
                println!("  Converted {} to JPEG for SSDV encoding", image);
                buf.into_inner()
            };

            // Validate callsign
            komitoto_ssdv::validate_callsign(&callsign)
                .map_err(|e| -> Box<dyn Error> { e.into() })?;

            let type_ = if no_fec { PacketType::Nofec } else { PacketType::Normal };

            let mut encoder = SsdvEncoder::new(type_, &callsign, id, quality, pkt_size)
                .map_err(|e| -> Box<dyn Error> { e.into() })?;

            encoder.feed(&jpeg_data);

            let bin_path = output.unwrap_or_else(|| {
                let base = std::path::Path::new(&image)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("ssdv_output");
                format!("{}.bin", base)
            });

            let mut all_packets = Vec::new();
            loop {
                match encoder.get_packet()? {
                    Some(packet) => all_packets.extend_from_slice(&packet),
                    None => break,
                }
            }

            std::fs::write(&bin_path, &all_packets)?;

            let num_packets = all_packets.len() / pkt_size;
            println!("SSDV encoded: {}", bin_path);
            println!("  Callsign: {}", callsign);
            println!("  Image ID: {}", id);
            println!("  Quality: {}", quality);
            println!("  Type: {}", if no_fec { "NOFEC" } else { "NORMAL" });
            println!("  Packets: {}", num_packets);
            println!("  Image: {}", image);
        }

        SsdvAction::Decode { input, output, verbose, drop, pkt_size } => {
            if !std::path::Path::new(&input).exists() {
                return Err(format!("Input file not found: {}", input).into());
            }

            let data = std::fs::read(&input)?;
            if data.len() < pkt_size {
                return Err("Input file too small for even one packet".into());
            }

            let num_packets = data.len() / pkt_size;
            let drop_pct = drop.unwrap_or(0).min(100) as f64 / 100.0;

            // Seed a simple RNG for drop test
            let mut rng_state: u32 = 42;
            let mut should_drop = || {
                if drop_pct <= 0.0 { return false; }
                rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
                let val = (rng_state >> 16) as f64 / 65536.0;
                val < drop_pct
            };

            let mut decoder = SsdvDecoder::new(pkt_size)
                .map_err(|e| -> Box<dyn Error> { e.into() })?;

            let mut done = false;
            let mut packets_fed = 0usize;
            let mut packets_dropped = 0usize;

            for i in 0..num_packets {
                let mut pkt = data[i * pkt_size..(i + 1) * pkt_size].to_vec();

                // Validate packet
                if let Ok(errors) = komitoto_ssdv::packet::validate_packet(&mut pkt, pkt_size) {
                    if should_drop() {
                        packets_dropped += 1;
                        if verbose {
                            eprintln!("Packet {} dropped (test)", i);
                        }
                        continue;
                    }

                    if verbose {
                        let info = komitoto_ssdv::packet::read_header(&pkt);
                        eprintln!("Packet {}: id={}, callsign={}, mcu_id={}, errors={}",
                            i, info.packet_id, info.callsign_s, info.mcu_id, errors);
                    }

                    packets_fed += 1;
                    match decoder.feed(&pkt) {
                        Ok(true) => { done = true; break; }
                        Ok(false) => {}
                        Err(e) => {
                            if verbose { eprintln!("Packet {} decode error: {}", i, e); }
                        }
                    }
                } else if verbose {
                    eprintln!("Packet {}: invalid, skipping", i);
                }
            }

            let jpeg_data = decoder.get_jpeg();

            let img_path = output.unwrap_or_else(|| {
                let base = std::path::Path::new(&input)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("decoded");
                format!("{}.jpeg", base)
            });

            // Write output - if non-JPEG format requested, convert via image crate
            let img_ext = std::path::Path::new(&img_path)
                .extension()
                .and_then(|e| e.to_str())
                .map(|e| e.to_lowercase())
                .unwrap_or_default();

            if img_ext == "jpeg" || img_ext == "jpg" {
                std::fs::write(&img_path, &jpeg_data)?;
            } else {
                let img = image::load_from_memory(&jpeg_data)
                    .map_err(|e| format!("Failed to load decoded JPEG for format conversion: {}", e))?;
                img.save(&img_path)
                    .map_err(|e| format!("Failed to save {}: {}", img_path, e))?;
            }

            println!("SSDV decoded: {}", img_path);
            println!("  Packets total: {}", num_packets);
            println!("  Packets fed: {}", packets_fed);
            if packets_dropped > 0 {
                println!("  Packets dropped (test): {}", packets_dropped);
            }
            if !done {
                println!("  Warning: image incomplete (missing packets)");
            }
        }

        SsdvAction::Info { input, pkt_size } => {
            if !std::path::Path::new(&input).exists() {
                return Err(format!("Input file not found: {}", input).into());
            }

            let data = std::fs::read(&input)?;
            let num_packets = data.len() / pkt_size;

            println!("File: {}", input);
            println!("Size: {} bytes", data.len());
            println!("Packets: {} (pkt_size={})", num_packets, pkt_size);

            // Show first valid packet info
            for i in 0..num_packets {
                let mut pkt = data[i * pkt_size..(i + 1) * pkt_size].to_vec();
                if let Ok(errors) = komitoto_ssdv::packet::validate_packet(&mut pkt, pkt_size) {
                    let info = komitoto_ssdv::packet::read_header(&pkt);
                    println!("\nFirst valid packet (#{})", i);
                    println!("  Type: {:?}", info.packet_type);
                    println!("  Callsign: {}", info.callsign_s);
                    println!("  Callsign code: {}", info.callsign);
                    println!("  Image ID: {}", info.image_id);
                    println!("  Packet ID: {}", info.packet_id);
                    println!("  Resolution: {}x{}", info.width, info.height);
                    println!("  Quality: {}", info.quality);
                    println!("  MCU mode: {}", info.mcu_mode);
                    println!("  MCU count: {}", info.mcu_count);
                    println!("  EOI: {}", info.eoi);
                    println!("  RS errors corrected: {}", errors);
                    break;
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
