use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Band {
    M2190, M630, M560, M160, M80, M60, M40, M30, M20, M17,
    M15, M12, M10, M8, M6, M5, M4, M2, M1_25,
    Cm70, Cm33, Cm23, Cm13, Cm9, Cm6, Cm3, Cm1_25,
    Mm6, Mm4, Mm2_5, Mm2, Mm1, Submm,
}

pub struct BandInfo {
    pub band: Band,
    pub lower_mhz: f64,
    pub upper_mhz: f64,
}

pub const ALL_BANDS: &[BandInfo] = &[
    BandInfo { band: Band::M2190, lower_mhz: 0.1357,   upper_mhz: 0.1378 },
    BandInfo { band: Band::M630,  lower_mhz: 0.472,    upper_mhz: 0.479 },
    BandInfo { band: Band::M560,  lower_mhz: 0.501,    upper_mhz: 0.504 },
    BandInfo { band: Band::M160,  lower_mhz: 1.8,      upper_mhz: 2.0 },
    BandInfo { band: Band::M80,   lower_mhz: 3.5,      upper_mhz: 4.0 },
    BandInfo { band: Band::M60,   lower_mhz: 5.06,     upper_mhz: 5.45 },
    BandInfo { band: Band::M40,   lower_mhz: 7.0,      upper_mhz: 7.3 },
    BandInfo { band: Band::M30,   lower_mhz: 10.1,     upper_mhz: 10.15 },
    BandInfo { band: Band::M20,   lower_mhz: 14.0,     upper_mhz: 14.35 },
    BandInfo { band: Band::M17,   lower_mhz: 18.068,   upper_mhz: 18.168 },
    BandInfo { band: Band::M15,   lower_mhz: 21.0,     upper_mhz: 21.45 },
    BandInfo { band: Band::M12,   lower_mhz: 24.890,   upper_mhz: 24.99 },
    BandInfo { band: Band::M10,   lower_mhz: 28.0,     upper_mhz: 29.7 },
    BandInfo { band: Band::M8,    lower_mhz: 40.0,     upper_mhz: 45.0 },
    BandInfo { band: Band::M6,    lower_mhz: 50.0,     upper_mhz: 54.0 },
    BandInfo { band: Band::M5,    lower_mhz: 54.000001,upper_mhz: 69.9 },
    BandInfo { band: Band::M4,    lower_mhz: 70.0,     upper_mhz: 71.0 },
    BandInfo { band: Band::M2,    lower_mhz: 144.0,    upper_mhz: 148.0 },
    BandInfo { band: Band::M1_25, lower_mhz: 222.0,    upper_mhz: 225.0 },
    BandInfo { band: Band::Cm70,  lower_mhz: 420.0,    upper_mhz: 450.0 },
    BandInfo { band: Band::Cm33,  lower_mhz: 902.0,    upper_mhz: 928.0 },
    BandInfo { band: Band::Cm23,  lower_mhz: 1240.0,   upper_mhz: 1300.0 },
    BandInfo { band: Band::Cm13,  lower_mhz: 2300.0,   upper_mhz: 2450.0 },
    BandInfo { band: Band::Cm9,   lower_mhz: 3300.0,   upper_mhz: 3500.0 },
    BandInfo { band: Band::Cm6,   lower_mhz: 5650.0,   upper_mhz: 5925.0 },
    BandInfo { band: Band::Cm3,   lower_mhz: 10000.0,  upper_mhz: 10500.0 },
    BandInfo { band: Band::Cm1_25,lower_mhz: 24000.0,  upper_mhz: 24250.0 },
    BandInfo { band: Band::Mm6,   lower_mhz: 47000.0,  upper_mhz: 47200.0 },
    BandInfo { band: Band::Mm4,   lower_mhz: 75500.0,  upper_mhz: 81000.0 },
    BandInfo { band: Band::Mm2_5, lower_mhz: 119980.0, upper_mhz: 123000.0 },
    BandInfo { band: Band::Mm2,   lower_mhz: 134000.0, upper_mhz: 149000.0 },
    BandInfo { band: Band::Mm1,   lower_mhz: 241000.0, upper_mhz: 250000.0 },
    BandInfo { band: Band::Submm, lower_mhz: 300000.0, upper_mhz: 7500000.0 },
];

impl Band {
    pub fn from_freq_mhz(freq: f64) -> Option<Band> {
        for info in ALL_BANDS {
            if freq >= info.lower_mhz && freq <= info.upper_mhz {
                return Some(info.band);
            }
        }
        None
    }

    pub fn name(&self) -> &'static str {
        match self {
            Band::M2190 => "2190m",
            Band::M630  => "630m",
            Band::M560  => "560m",
            Band::M160  => "160m",
            Band::M80   => "80m",
            Band::M60   => "60m",
            Band::M40   => "40m",
            Band::M30   => "30m",
            Band::M20   => "20m",
            Band::M17   => "17m",
            Band::M15   => "15m",
            Band::M12   => "12m",
            Band::M10   => "10m",
            Band::M8    => "8m",
            Band::M6    => "6m",
            Band::M5    => "5m",
            Band::M4    => "4m",
            Band::M2    => "2m",
            Band::M1_25 => "1.25m",
            Band::Cm70  => "70cm",
            Band::Cm33  => "33cm",
            Band::Cm23  => "23cm",
            Band::Cm13  => "13cm",
            Band::Cm9   => "9cm",
            Band::Cm6   => "6cm",
            Band::Cm3   => "3cm",
            Band::Cm1_25=> "1.25cm",
            Band::Mm6   => "6mm",
            Band::Mm4   => "4mm",
            Band::Mm2_5 => "2.5mm",
            Band::Mm2   => "2mm",
            Band::Mm1   => "1mm",
            Band::Submm => "submm",
        }
    }

    pub fn from_name(name: &str) -> Option<Band> {
        match name.to_uppercase().as_str() {
            "2190M" => Some(Band::M2190),
            "630M"  => Some(Band::M630),
            "560M"  => Some(Band::M560),
            "160M"  => Some(Band::M160),
            "80M"   => Some(Band::M80),
            "60M"   => Some(Band::M60),
            "40M"   => Some(Band::M40),
            "30M"   => Some(Band::M30),
            "20M"   => Some(Band::M20),
            "17M"   => Some(Band::M17),
            "15M"   => Some(Band::M15),
            "12M"   => Some(Band::M12),
            "10M"   => Some(Band::M10),
            "8M"    => Some(Band::M8),
            "6M"    => Some(Band::M6),
            "5M"    => Some(Band::M5),
            "4M"    => Some(Band::M4),
            "2M"    => Some(Band::M2),
            "1.25M" => Some(Band::M1_25),
            "70CM"  => Some(Band::Cm70),
            "33CM"  => Some(Band::Cm33),
            "23CM"  => Some(Band::Cm23),
            "13CM"  => Some(Band::Cm13),
            "9CM"   => Some(Band::Cm9),
            "6CM"   => Some(Band::Cm6),
            "3CM"   => Some(Band::Cm3),
            "1.25CM"=> Some(Band::Cm1_25),
            "6MM"   => Some(Band::Mm6),
            "4MM"   => Some(Band::Mm4),
            "2.5MM" => Some(Band::Mm2_5),
            "2MM"   => Some(Band::Mm2),
            "1MM"   => Some(Band::Mm1),
            "SUBMM" => Some(Band::Submm),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    AM,
    ATV,
    CHIP,
    CW,
    DIGITALVOICE,
    FAX,
    FM,
    FSK,
    FT8,
    MFSK,
    MFDM,
    PAC,
    PAX,
    PSK,
    ROS,
    RTTY,
    SSB,
    SSTV,
}

impl Mode {
    pub fn from_str(s: &str) -> Option<Mode> {
        match s.to_uppercase().as_str() {
            "AM"    => Some(Mode::AM),
            "ATV"   => Some(Mode::ATV),
            "CHIP"  => Some(Mode::CHIP),
            "CW"    => Some(Mode::CW),
            "DIGITALVOICE" | "DV" => Some(Mode::DIGITALVOICE),
            "FAX"   => Some(Mode::FAX),
            "FM"    => Some(Mode::FM),
            "FSK"   => Some(Mode::FSK),
            "FT8"   => Some(Mode::FT8),
            "MFSK"  => Some(Mode::MFSK),
            "MFDM"  => Some(Mode::MFDM),
            "PAC"   => Some(Mode::PAC),
            "PAX"   => Some(Mode::PAX),
            "PSK"   => Some(Mode::PSK),
            "ROS"   => Some(Mode::ROS),
            "RTTY"  => Some(Mode::RTTY),
            "SSB"   => Some(Mode::SSB),
            "SSTV"  => Some(Mode::SSTV),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Mode::AM => "AM",
            Mode::ATV => "ATV",
            Mode::CHIP => "CHIP",
            Mode::CW => "CW",
            Mode::DIGITALVOICE => "DIGITALVOICE",
            Mode::FAX => "FAX",
            Mode::FM => "FM",
            Mode::FSK => "FSK",
            Mode::FT8 => "FT8",
            Mode::MFSK => "MFSK",
            Mode::MFDM => "MFDM",
            Mode::PAC => "PAC",
            Mode::PAX => "PAX",
            Mode::PSK => "PSK",
            Mode::ROS => "ROS",
            Mode::RTTY => "RTTY",
            Mode::SSB => "SSB",
            Mode::SSTV => "SSTV",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QslRcvd {
    Y,
    N,
    R,
    I,
    V,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QslSent {
    Y,
    N,
    R,
    Q,
    I,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QslVia {
    B,
    D,
    E,
    M,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QsoComplete {
    Y,
    N,
    NIL,
    UNCERTAIN,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Qso {
    pub id: String,
    pub date_time_on: DateTime<Utc>,
    pub call: String,
    pub freq: f64,
    pub mode: Mode,
    pub rst_sent: Option<String>,
    pub rst_rcvd: Option<String>,
    pub date_time_off: Option<DateTime<Utc>>,
    pub qth: Option<String>,
    pub rig: Option<String>,
    pub address: Option<String>,
    pub age: Option<u8>,
    pub altitude: Option<i32>,
    pub band: Option<Band>,
    pub email: Option<String>,
    pub lat: Option<f64>,
    pub lon: Option<f64>,
    pub my_altitude: Option<i32>,
    pub my_antenna: Option<String>,
    pub my_city: Option<String>,
    pub my_country: Option<String>,
    pub my_lat: Option<f64>,
    pub my_lon: Option<f64>,
    pub my_name: Option<String>,
    pub my_rig: Option<String>,
    pub name: Option<String>,
    pub operator: Option<String>,
    pub owner_callsign: Option<String>,
    pub qsl_r_date: Option<DateTime<Utc>>,
    pub qsl_s_date: Option<DateTime<Utc>>,
    pub qsl_rcvd: Option<QslRcvd>,
    pub qsl_sent: Option<QslSent>,
    pub qsl_rcvd_via: Option<QslVia>,
    pub qsl_sent_via: Option<QslVia>,
    pub tx_power: Option<u8>,
    pub rx_power: Option<u8>,
    pub qso_complete: Option<QsoComplete>,
    pub grid: Option<String>,
    pub my_grid: Option<String>,
    pub comment: Option<String>,
}

impl Qso {
    pub fn new(call: String, freq: f64, mode: Mode, date_time_on: DateTime<Utc>) -> Self {
        let band = Band::from_freq_mhz(freq);
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            date_time_on,
            call,
            freq,
            mode,
            rst_sent: None,
            rst_rcvd: None,
            date_time_off: None,
            qth: None,
            rig: None,
            address: None,
            age: None,
            altitude: None,
            band,
            email: None,
            lat: None,
            lon: None,
            my_altitude: None,
            my_antenna: None,
            my_city: None,
            my_country: None,
            my_lat: None,
            my_lon: None,
            my_name: None,
            my_rig: None,
            name: None,
            operator: None,
            owner_callsign: None,
            qsl_r_date: None,
            qsl_s_date: None,
            qsl_rcvd: None,
            qsl_sent: None,
            qsl_rcvd_via: None,
            qsl_sent_via: None,
            tx_power: None,
            rx_power: None,
            qso_complete: None,
            grid: None,
            my_grid: None,
            comment: None,
        }
    }
}
