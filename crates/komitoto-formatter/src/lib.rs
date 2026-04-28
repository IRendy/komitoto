use chrono::{DateTime, Utc, TimeZone};
use komitoto_types::Qso;
use komitoto_calc::sunrise::SunTimes;

/// Formatter for sunrise calculation results
pub struct SunriseFormatter;

impl SunriseFormatter {
    /// Format sunrise data as table with both UTC and local time
    pub fn format_sunrise(
        date: &str,
        lat: f64,
        lon: f64,
        altitude: f64,
        times: &SunTimes,
        use_json: bool,
    ) -> Result<String, Box<dyn std::error::Error>> {
        if use_json {
            Ok(Self::format_sunrise_json(date, lat, lon, altitude, times))
        } else {
            Ok(Self::format_sunrise_text(date, lat, lon, altitude, times))
        }
    }

    fn format_sunrise_text(
        date: &str,
        lat: f64,
        lon: f64,
        altitude: f64,
        times: &SunTimes,
    ) -> String {
        let mut output = String::new();
        
        // Use Asia/Shanghai timezone directly
        let shanghai = chrono_tz::Asia::Shanghai;
        
        output.push_str(&format!(
            "Sun times for {} at ({}, {}) alt={}m\n",
            date, lat, lon, altitude
        ));
        output.push_str("==================================================\n");
        output.push_str(&format!(
            "  Sunrise:  {} (BJT: {})\n",
            times.sunrise.format("%H:%M:%S UTC"),
            times.sunrise.with_timezone(&shanghai).format("%Y-%m-%d %H:%M:%S")
        ));
        output.push_str(&format!(
            "  Sunset:   {} (BJT: {})\n",
            times.sunset.format("%H:%M:%S UTC"),
            times.sunset.with_timezone(&shanghai).format("%Y-%m-%d %H:%M:%S")
        ));
        output.push_str(&format!(
            "  Dawn:     {} (BJT: {})\n",
            times.dawn.format("%H:%M:%S UTC"),
            times.dawn.with_timezone(&shanghai).format("%Y-%m-%d %H:%M:%S")
        ));
        output.push_str(&format!(
            "  Dusk:     {} (BJT: {})\n",
            times.dusk.format("%H:%M:%S UTC"),
            times.dusk.with_timezone(&shanghai).format("%Y-%m-%d %H:%M:%S")
        ));
        
        output
    }

    fn format_sunrise_json(
        date: &str,
        lat: f64,
        lon: f64,
        altitude: f64,
        times: &SunTimes,
    ) -> String {
        format!(
            "{{
  \"date\": \"{}\",
  \"lat\": {},
  \"lon\": {},
  \"altitude\": {},
  \"sunrise\": \"{}\",
  \"sunset\": \"{}\",
  \"dawn\": \"{}\",
  \"dusk\": \"{}\"
}}",
            date,
            lat,
            lon,
            altitude,
            times.sunrise.to_rfc3339(),
            times.sunset.to_rfc3339(),
            times.dawn.to_rfc3339(),
            times.dusk.to_rfc3339()
        )
    }
}

/// QSO List and Detail Formatter
pub struct QsoFormatter;

impl QsoFormatter {
    /// Format QSO list as table
    pub fn format_qso_list(qsos: &[Qso]) -> String {
        if qsos.is_empty() {
            return "No QSOs found.".to_string();
        }

        let mut output = String::new();
        output.push_str(&format!(
            "{:<36} {:<10} {:>10} {:<6} {:<5} {:<5} {:<8} {}\n",
            "ID", "Call", "Freq", "Mode", "S", "R", "Grid", "Time (UTC)"
        ));
        output.push_str(&"-".repeat(100));
        output.push('\n');

        for qso in qsos {
            let id_short: String = qso.id.chars().take(8).collect();
            output.push_str(&format!(
                "{:<36} {:<10} {:>10.3} {:<6} {:<5} {:<5} {:<8} {}\n",
                id_short,
                qso.call,
                qso.freq,
                qso.mode.as_str(),
                qso.rst_sent.as_deref().unwrap_or("-"),
                qso.rst_rcvd.as_deref().unwrap_or("-"),
                qso.grid.as_deref().unwrap_or("-"),
                qso.date_time_on.format("%Y-%m-%d %H:%M")
            ));
        }

        output.push_str(&format!("\n{} QSO(s) shown.", qsos.len()));
        output
    }

    /// Format single QSO detail
    pub fn format_qso_detail(qso: &Qso) -> String {
        let mut output = String::new();
        output.push_str("QSO Detail\n");
        output.push_str(&"=".repeat(50));
        output.push('\n');
        output.push_str(&format!("  ID:       {}\n", qso.id));
        output.push_str(&format!("  Call:     {}\n", qso.call));
        output.push_str(&format!("  Freq:     {:.3} MHz\n", qso.freq));
        output.push_str(&format!("  Band:     {}\n", 
            qso.band.map(|b| b.name()).unwrap_or("-")));
        output.push_str(&format!("  Mode:     {}\n", qso.mode.as_str()));
        output.push_str(&format!("  Date/On:  {}\n", qso.date_time_on.format("%Y-%m-%d %H:%M:%S UTC")));
        
        if let Some(off) = qso.date_time_off {
            output.push_str(&format!("  Date/Off: {}\n", off.format("%Y-%m-%d %H:%M:%S UTC")));
        }
        
        output.push_str(&format!(
            "  RST S/R:  {}/{}\n",
            qso.rst_sent.as_deref().unwrap_or("-"),
            qso.rst_rcvd.as_deref().unwrap_or("-")
        ));

        if let Some(ref v) = qso.grid {
            output.push_str(&format!("  Grid:     {}\n", v));
        }
        if let Some(ref v) = qso.qth {
            output.push_str(&format!("  QTH:      {}\n", v));
        }
        if let Some(ref v) = qso.rig {
            output.push_str(&format!("  Rig:      {}\n", v));
        }
        if let Some(ref v) = qso.name {
            output.push_str(&format!("  Name:     {}\n", v));
        }
        if let Some(ref v) = qso.comment {
            output.push_str(&format!("  Comment:  {}\n", v));
        }

        output
    }
}

/// Parse sunrise/dawn type string
pub fn parse_dawn_type(dawn_str: Option<&str>) -> Result<Option<sunrise::DawnType>, String> {
    match dawn_str {
        Some("civil") => Ok(Some(sunrise::DawnType::Civil)),
        Some("nautical") => Ok(Some(sunrise::DawnType::Nautical)),
        Some("astronomical") => Ok(Some(sunrise::DawnType::Astronomical)),
        Some(other) => Err(format!(
            "Unknown dawn type: {}. Use civil, nautical, or astronomical.",
            other
        )),
        None => Ok(None),
    }
}

/// Parse datetime from date and time strings
pub fn parse_datetime(
    date_str: &str,
    time_str: Option<&str>,
) -> Result<DateTime<Utc>, Box<dyn std::error::Error>> {
    let nd = chrono::NaiveDate::parse_from_str(date_str, "%Y%m%d")?;
    
    let nt = match time_str {
        Some(t) => {
            match t.len() {
                6 => chrono::NaiveTime::parse_from_str(t, "%H%M%S")?,
                4 => chrono::NaiveTime::parse_from_str(t, "%H%M")?,
                _ => return Err(format!(
                    "Invalid time format: {}. Use HHMMSS or HHMM.",
                    t
                ).into()),
            }
        }
        None => chrono::NaiveTime::from_hms_opt(0, 0, 0).unwrap(),
    };

    let dt = nd.and_time(nt);
    Ok(Utc.from_utc_datetime(&dt))
}
