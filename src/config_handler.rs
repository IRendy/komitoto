/// Configuration management for komitoto

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct ConfigFile {
    pub callsign: Option<String>,
    pub name: Option<String>,
    pub qth: Option<String>,
    pub country: Option<String>,
    pub rig: Option<String>,
    pub rx: Option<String>,
    pub grid: Option<String>,
    #[serde(default = "default_timezone")]
    pub timezone: String,
    pub my_altitude: Option<i32>,
    pub my_antenna: Option<String>,
    pub my_city: Option<String>,
    pub tx_power: Option<u8>,
    pub rx_power: Option<u8>,
    pub rst_sent_default: Option<String>,
    pub rst_rcvd_default: Option<String>,
}

fn default_timezone() -> String {
    "Asia/Shanghai".to_string()
}

impl ConfigFile {
    pub fn init_or_exists() -> Result<(), Box<dyn std::error::Error>> {
        let config_file = "komitoto.toml";
        
        if std::path::Path::new(config_file).exists() {
            println!("Configuration file already exists: {}", config_file);
            println!("\nEdit it manually or use 'komitoto config show' to view current values.");
            Ok(())
        } else {
            // Copy from example
            let example_file = "komitoto.toml.example";
            if std::path::Path::new(example_file).exists() {
                std::fs::copy(example_file, config_file)?;
                println!("Created {} from example file", config_file);
                println!("\nPlease edit {} and set your information.", config_file);
            } else {
                // Create with defaults
                let config = Self::default();
                let content = toml::to_string_pretty(&config)?;
                std::fs::write(config_file, content)?;
                println!("Created {} with default values", config_file);
            }
            Ok(())
        }
    }
    
    pub fn show() -> Result<(), Box<dyn std::error::Error>> {
        let config_file = "komitoto.toml";
        
        if !std::path::Path::new(config_file).exists() {
            println!("No configuration file found. Run 'komitoto config init' to create one.");
            return Ok(());
        }
        
        let content = std::fs::read_to_string(config_file)?;
        println!("Current configuration ({}):", config_file);
        println!("{}", "=".repeat(50));
        println!("{}", content);
        Ok(())
    }
    
    pub fn set(field: &str, value: &str) -> Result<(), Box<dyn std::error::Error>> {
        let config_file = "komitoto.toml";
        
        if !std::path::Path::new(config_file).exists() {
            println!("Configuration file not found. Run 'komitoto config init' first.");
            return Ok(());
        }
        
        let content = std::fs::read_to_string(config_file)?;
        
        let updated = match field {
            "callsign" => format!("{}\ncallsign = \"{}\"", 
                remove_line(&content, "callsign"),
                escape_toml_string(value)),
            "name" => format!("{}\nname = \"{}\"",
                remove_line(&content, "name"),
                escape_toml_string(value)),
            "qth" => format!("{}\nqth = \"{}\"",
                remove_line(&content, "qth"),
                escape_toml_string(value)),
            "country" => format!("{}\ncountry = \"{}\"",
                remove_line(&content, "country"),
                escape_toml_string(value)),
            "rig" => format!("{}\nrig = \"{}\"",
                remove_line(&content, "rig"),
                escape_toml_string(value)),
            "rx" => format!("{}\nrx = \"{}\"",
                remove_line(&content, "rx"),
                escape_toml_string(value)),
            "grid" => format!("{}\ngrid = \"{}\"",
                remove_line(&content, "grid"),
                escape_toml_string(value)),
            "timezone" => format!("{}\ntimezone = \"{}\"",
                remove_line(&content, "timezone"),
                escape_toml_string(value)),
            "my_altitude" => format!("{}\nmy_altitude = {}",
                remove_line(&content, "my_altitude"),
                value.parse::<i32>().unwrap_or(0)),
            "my_antenna" => format!("{}\nmy_antenna = \"{}\"",
                remove_line(&content, "my_antenna"),
                escape_toml_string(value)),
            "my_city" => format!("{}\nmy_city = \"{}\"",
                remove_line(&content, "my_city"),
                escape_toml_string(value)),
            "tx_power" => format!("{}\ntx_power = {}",
                remove_line(&content, "tx_power"),
                value.parse::<u8>().unwrap_or(100)),
            "rx_power" => format!("{}\nrx_power = {}",
                remove_line(&content, "rx_power"),
                value.parse::<u8>().unwrap_or(0)),
            "rst_sent_default" => format!("{}\nrst_sent_default = \"{}\"",
                remove_line(&content, "rst_sent_default"),
                escape_toml_string(value)),
            "rst_rcvd_default" => format!("{}\nrst_rcvd_default = \"{}\"",
                remove_line(&content, "rst_rcvd_default"),
                escape_toml_string(value)),
            _ => {
                println!("Unknown field: {}. Available fields:", field);
                println!("  callsign, name, qth, country, rig, rx, grid, timezone");
                println!("  my_altitude, my_antenna, my_city, tx_power, rx_power");
                println!("  rst_sent_default, rst_rcvd_default");
                return Ok(());
            }
        };
        
        std::fs::write(config_file, updated)?;
        println!("Updated {}: {}", field, value);
        Ok(())
    }
}

fn remove_line(content: &str, field: &str) -> String {
    content.lines()
        .filter(|line| !line.trim().starts_with(field))
        .collect::<Vec<_>>()
        .join("\n")
        .trim_end()
        .to_string()
}

fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}
