use serde::{Deserialize, Serialize};

/// User settings configuration file
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserProfile {
    /// Your callsign
    #[serde(default)]
    pub callsign: Option<String>,
    
    /// Your name (nickname)
    #[serde(default)]
    pub name: Option<String>,
    
    /// Your QTH (location description)
    #[serde(default)]
    pub qth: Option<String>,
    
    /// Your country code (e.g., "China", "US", "JP")
    #[serde(default)]
    pub country: Option<String>,
    
    /// Your equipment/rig
    #[serde(default)]
    pub rig: Option<String>,
    
    /// Your receiver
    #[serde(default)]
    pub rx: Option<String>,
    
    /// Your grid square (your location)
    #[serde(default)]
    pub grid: Option<String>,
    
    /// Timezone (IANA timezone name)
    #[serde(default = "default_timezone")]
    pub timezone: String,
    
    /// My altitude (meters above sea level)
    #[serde(default)]
    pub my_altitude: Option<i32>,
    
    /// My antenna type/description
    #[serde(default)]
    pub my_antenna: Option<String>,
    
    /// My city/location
    #[serde(default)]
    pub my_city: Option<String>,
    
    /// Transmit power (watts)
    #[serde(default)]
    pub tx_power: Option<u8>,
    
    /// Receive power (watts or - for dBm)
    #[serde(default)]
    pub rx_power: Option<u8>,
    
    /// RST sent default value
    #[serde(default)]
    pub rst_sent_default: Option<String>,
    
    /// RST received default value
    #[serde(default)]
    pub rst_rcvd_default: Option<String>,
}

fn default_timezone() -> String {
    "Asia/Shanghai".to_string()
}
