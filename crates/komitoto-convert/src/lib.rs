use komitoto_types::*;
use chrono::{Datelike, TimeZone, Timelike};
use rusqlite::{params, Connection};

const ADIF_VER: &str = "3.1.7";
const PROGRAMID: &str = "komitoto";

/// Convert a list of QSOs to ADIF format string
pub fn qsos_to_adi(qsos: &[Qso]) -> String {
    let mut out = String::new();

    // ADIF header
    out.push_str(&format!("ADIF_VER={}\n", adif_ver_str()));
    out.push_str(&adi_field("programid", PROGRAMID));
    out.push_str(&adi_field("adif_ver", ADIF_VER));
    out.push_str("<eoh>\n\n");

    for qso in qsos {
        out.push_str(&qso_to_adi(qso));
        out.push_str("<eor>\n\n");
    }

    out
}

fn adif_ver_str() -> String {
    ADIF_VER.replace('.', "_")
}

fn adi_field(name: &str, value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    format!("<{}:{}>{}", name.to_uppercase(), value.len(), value)
}

fn adi_field_opt(name: &str, value: &Option<String>) -> String {
    match value {
        Some(v) if !v.is_empty() => adi_field(name, v),
        _ => String::new(),
    }
}

fn qso_to_adi(qso: &Qso) -> String {
    let mut fields = Vec::new();

    // QSO_DATE: YYYYMMDD
    let date_on = qso.date_time_on;
    fields.push(adi_field("qso_date", &format!(
        "{:04}{:02}{:02}", date_on.year(), date_on.month(), date_on.day()
    )));
    // TIME_ON: HHMMSS
    fields.push(adi_field("time_on", &format!(
        "{:02}{:02}{:02}", date_on.hour(), date_on.minute(), date_on.second()
    )));
    fields.push(adi_field("call", &qso.call));
    fields.push(adi_field("freq", &format!("{:.6}", qso.freq)));
    fields.push(adi_field("mode", qso.mode.as_str()));

    if let Some(band) = qso.band {
        fields.push(adi_field("band", band.name()));
    }

    fields.push(adi_field_opt("rst_sent", &qso.rst_sent));
    fields.push(adi_field_opt("rst_rcvd", &qso.rst_rcvd));
    fields.push(adi_field_opt("qth", &qso.qth));
    fields.push(adi_field_opt("rig", &qso.rig));
    fields.push(adi_field_opt("address", &qso.address));
    fields.push(adi_field_opt("email", &qso.email));
    fields.push(adi_field_opt("name", &qso.name));
    fields.push(adi_field_opt("operator", &qso.operator));
    fields.push(adi_field_opt("grid", &qso.grid));
    fields.push(adi_field_opt("my_grid", &qso.my_grid));
    fields.push(adi_field_opt("my_name", &qso.my_name));
    fields.push(adi_field_opt("my_rig", &qso.my_rig));
    fields.push(adi_field_opt("my_city", &qso.my_city));
    fields.push(adi_field_opt("my_country", &qso.my_country));
    fields.push(adi_field_opt("comment", &qso.comment));

    if let Some(dt_off) = qso.date_time_off {
        fields.push(adi_field("qso_date_off", &format!(
            "{:04}{:02}{:02}", dt_off.year(), dt_off.month(), dt_off.day()
        )));
        fields.push(adi_field("time_off", &format!(
            "{:02}{:02}{:02}", dt_off.hour(), dt_off.minute(), dt_off.second()
        )));
    }

    if let Some(age) = qso.age {
        fields.push(adi_field("age", &age.to_string()));
    }
    if let Some(alt) = qso.altitude {
        fields.push(adi_field("altitude", &alt.to_string()));
    }
    if let Some(lat) = qso.lat {
        fields.push(adi_field("lat", &format!("{:.4}", lat)));
    }
    if let Some(lon) = qso.lon {
        fields.push(adi_field("lon", &format!("{:.4}", lon)));
    }
    if let Some(pwr) = qso.tx_power {
        fields.push(adi_field("tx_pwr", &pwr.to_string()));
    }

    fields.join("")
}

/// Convert a list of QSOs to JSON format string
pub fn qsos_to_json(qsos: &[Qso]) -> String {
    serde_json::to_string_pretty(qsos).unwrap_or_else(|e| format!("{{\"error\": \"{}\"}}", e))
}

/// Parse JSON string into QSOs
pub fn json_to_qsos(json_str: &str) -> Result<Vec<Qso>, serde_json::Error> {
    serde_json::from_str(json_str)
}

/// Parse ADIF string into QSOs (basic implementation)
pub fn adi_to_qsos(adi: &str) -> Vec<Qso> {
    let mut qsos = Vec::new();

    // Find end of header
    let body = match adi.find("<eoh>") {
        Some(pos) => &adi[pos + 5..],
        None => adi,
    };

    let records: Vec<&str> = body.split("<eor>").collect();

    for record in records {
        let record = record.trim();
        if record.is_empty() {
            continue;
        }

        let mut call = String::new();
        let mut freq = 0.0;
        let mut mode = Mode::CW;
        let mut date_str = String::new();
        let mut time_str = String::new();
        let mut rst_sent = None;
        let mut rst_rcvd = None;
        let mut band = None;
        let mut grid = None;
        let mut qth = None;
        let mut rig = None;
        let mut name = None;
        let mut comment = None;

        let mut pos = 0;
        let bytes = record.as_bytes();
        while pos < bytes.len() {
            // Find field tag start
            if bytes[pos] != b'<' {
                pos += 1;
                continue;
            }
            // Parse tag: <NAME:LENGTH>
            let tag_start = pos + 1;
            if let Some(tag_end) = record[pos..].find('>') {
                let tag_content = &record[tag_start..pos + tag_end];
                pos += tag_end + 1;

                // Parse field name and length
                let parts: Vec<&str> = tag_content.split(':').collect();
                let field_name = parts[0].to_lowercase();
                let data_len: usize = if parts.len() > 1 {
                    parts[1].split(':').next().unwrap_or("0").parse().unwrap_or(0)
                } else {
                    0
                };

                let value = if data_len > 0 && pos + data_len <= bytes.len() {
                    let v = record[pos..pos + data_len].to_string();
                    pos += data_len;
                    v
                } else {
                    String::new()
                };

                match field_name.as_str() {
                    "call" => call = value,
                    "freq" => freq = value.parse().unwrap_or(0.0),
                    "mode" => mode = Mode::from_str(&value).unwrap_or(Mode::CW),
                    "qso_date" => date_str = value,
                    "time_on" => time_str = value,
                    "rst_sent" => rst_sent = Some(value),
                    "rst_rcvd" => rst_rcvd = Some(value),
                    "band" => band = Band::from_name(&value),
                    "gridsquare" | "grid" => grid = Some(value),
                    "qth" => qth = Some(value),
                    "rig" => rig = Some(value),
                    "name" => name = Some(value),
                    "comment" => comment = Some(value),
                    _ => {}
                }
            } else {
                pos += 1;
            }
        }
        
        // Trim trailing special characters from call (like < or >)
        call = call.trim_matches(|c| c == '<' || c == '>').to_string();

        if call.is_empty() {
            continue;
        }

        let date_time_on = parse_adif_datetime(&date_str, &time_str);
        if band.is_none() && freq > 0.0 {
            band = Band::from_freq_mhz(freq);
        }

        let mut qso = Qso::new(call, freq, mode, date_time_on);
        qso.rst_sent = rst_sent;
        qso.rst_rcvd = rst_rcvd;
        qso.band = band;
        qso.grid = grid;
        qso.qth = qth;
        qso.rig = rig;
        qso.name = name;
        qso.comment = comment;
        qsos.push(qso);
    }

    qsos
}

fn parse_adif_datetime(date: &str, time: &str) -> chrono::DateTime<chrono::Utc> {
    use chrono::{NaiveDate, NaiveDateTime, NaiveTime};

    let nd = if date.len() >= 8 {
        NaiveDate::parse_from_str(date, "%Y%m%d").ok()
    } else {
        None
    };

    let nt = if time.len() >= 6 {
        NaiveTime::parse_from_str(time, "%H%M%S").ok()
    } else if time.len() >= 4 {
        NaiveTime::parse_from_str(&format!("{}00", time), "%H%M%S").ok()
    } else {
        None
    };

    let dt = match (nd, nt) {
        (Some(d), Some(t)) => Some(NaiveDateTime::new(d, t)),
        (Some(d), None) => Some(d.and_hms_opt(0, 0, 0).unwrap()),
        _ => None,
    };

    match dt {
        Some(d) => chrono::TimeZone::from_utc_datetime(&chrono::Utc, &d),
        None => chrono::Utc::now(),
    }
}

/// Parse CSV (as in qsos.csv) into QSOs
pub fn csv_to_qsos(csv: &str) -> Vec<Qso> {
    let mut qsos = Vec::new();
    let mut lines = csv.lines();

    // Skip header
    lines.next();

    for line in lines {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() < 5 {
            continue;
        }

        let call = fields[0].to_string();
        let freq: f64 = fields[1].parse().unwrap_or(0.0);
        let band = Band::from_name(fields[2]).or_else(|| Band::from_freq_mhz(freq));
        let mode = Mode::from_str(fields[3]).unwrap_or(Mode::CW);

        // Parse BJT time: 2025-05-14 21:44:15
        let date_time_on = chrono::NaiveDateTime::parse_from_str(fields[4], "%Y-%m-%d %H:%M:%S")
            .ok()
            .map(|dt| {
                chrono::FixedOffset::east_opt(8 * 3600)
                    .unwrap()
                    .from_utc_datetime(&dt)
                    .to_utc()
            })
            .unwrap_or_else(chrono::Utc::now);

        let grid = fields.get(5).and_then(|g| {
            if *g == "N/A" || g.is_empty() { None } else { Some(g.to_string()) }
        });
        let rig = fields.get(6).and_then(|r| {
            if r.is_empty() { None } else { Some(r.to_string()) }
        });

        let mut qso = Qso::new(call, freq, mode, date_time_on);
        qso.band = band;
        qso.grid = grid;
        qso.rig = rig;
        qsos.push(qso);
    }

    qsos
}

/// Convert QSOs to CSV string (extended format with all common fields)
pub fn qsos_to_csv(qsos: &[Qso]) -> String {
    let mut out = String::new();
    out.push_str("call,freq,band,mode,time_on,rst_sent,rst_rcvd,grid,qth,rig,name,comment,time_off\n");
    for qso in qsos {
        let band_str = qso.band.map(|b| b.name()).unwrap_or("");
        let time_on = qso.date_time_on.format("%Y-%m-%d %H:%M:%S").to_string();
        out.push_str(&format!(
            "{},{},{},{},{},{},{},{},{},{},{},{}\n",
            escape_csv(&qso.call),
            qso.freq,
            band_str,
            qso.mode.as_str(),
            time_on,
            escape_csv(&qso.rst_sent.as_deref().unwrap_or("")),
            escape_csv(&qso.rst_rcvd.as_deref().unwrap_or("")),
            escape_csv(&qso.grid.as_deref().unwrap_or("")),
            escape_csv(&qso.qth.as_deref().unwrap_or("")),
            escape_csv(&qso.rig.as_deref().unwrap_or("")),
            escape_csv(&qso.name.as_deref().unwrap_or("")),
            escape_csv(&qso.comment.as_deref().unwrap_or(""))
        ));
    }
    out
}

fn escape_csv(s: &str) -> String {
    if s.contains(',') || s.contains('"') || s.contains('\n') {
        let escaped = s.replace('"', "\"\"");
        format!("\"{}\"", escaped)
    } else {
        s.to_string()
    }
}

/// Convert QSOs to ADX (XML ADIF) format string
pub fn qsos_to_adx(qsos: &[Qso]) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");
    out.push_str("<ADX>\n");
    out.push_str("  <HEADER>\n");
    out.push_str(&format!("    <ADIF_VER>{}</ADIF_VER>\n", ADIF_VER));
    out.push_str(&format!("    <PROGRAMID>{}</PROGRAMID>\n", PROGRAMID));
    out.push_str("  </HEADER>\n");
    out.push_str("  <RECORDS>\n");
    for qso in qsos {
        out.push_str(&qso_to_adx(qso));
    }
    out.push_str("  </RECORDS>\n");
    out.push_str("</ADX>\n");
    out
}

fn adx_field(name: &str, value: &str) -> String {
    if value.is_empty() {
        return String::new();
    }
    format!("      <{}>{}</{}>\n", name.to_uppercase(), xml_escape(value), name.to_uppercase())
}

fn adx_field_opt(name: &str, value: &Option<String>) -> String {
    match value {
        Some(v) if !v.is_empty() => adx_field(name, v),
        _ => String::new(),
    }
}

fn xml_escape(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
}

fn qso_to_adx(qso: &Qso) -> String {
    let mut fields = Vec::new();
    let date_on = qso.date_time_on;
    fields.push(adx_field("CALL", &qso.call));
    fields.push(adx_field("FREQ", &format!("{:.6}", qso.freq)));
    fields.push(adx_field("MODE", qso.mode.as_str()));
    fields.push(adx_field("QSO_DATE", &format!(
        "{:04}{:02}{:02}", date_on.year(), date_on.month(), date_on.day()
    )));
    fields.push(adx_field("TIME_ON", &format!(
        "{:02}{:02}{:02}", date_on.hour(), date_on.minute(), date_on.second()
    )));
    if let Some(band) = qso.band {
        fields.push(adx_field("BAND", band.name()));
    }
    fields.push(adx_field_opt("RST_SENT", &qso.rst_sent));
    fields.push(adx_field_opt("RST_RCVD", &qso.rst_rcvd));
    fields.push(adx_field_opt("QTH", &qso.qth));
    fields.push(adx_field_opt("RIG", &qso.rig));
    fields.push(adx_field_opt("NAME", &qso.name));
    fields.push(adx_field_opt("GRIDSQUARE", &qso.grid));
    fields.push(adx_field_opt("COMMENT", &qso.comment));
    if let Some(dt_off) = qso.date_time_off {
        fields.push(adx_field("QSO_DATE_OFF", &format!(
            "{:04}{:02}{:02}", dt_off.year(), dt_off.month(), dt_off.day()
        )));
        fields.push(adx_field("TIME_OFF", &format!(
            "{:02}{:02}{:02}", dt_off.hour(), dt_off.minute(), dt_off.second()
        )));
    }
    format!("    <RECORD>\n{}    </RECORD>\n", fields.join(""))
}

/// Parse ADX (XML ADIF) string into QSOs
/// This is a simplified parser without external XML libraries.
pub fn adx_to_qsos(adx: &str) -> Vec<Qso> {
    let mut qsos = Vec::new();
    let records: Vec<&str> = adx.split("<RECORD>").skip(1).collect();

    for record in records {
        let record_end = record.find("</RECORD>").unwrap_or(record.len());
        let record = &record[..record_end];

        let mut call = String::new();
        let mut freq = 0.0;
        let mut mode = Mode::CW;
        let mut date_str = String::new();
        let mut time_str = String::new();
        let mut rst_sent = None;
        let mut rst_rcvd = None;
        let mut band = None;
        let mut grid = None;
        let mut qth = None;
        let mut rig = None;
        let mut name = None;
        let mut comment = None;

        let mut pos = 0;
        while pos < record.len() {
            if record.as_bytes().get(pos) != Some(&b'<') {
                pos += 1;
                continue;
            }
            let tag_start = pos + 1;
            if let Some(tag_end) = record[pos..].find('>') {
                let tag_name = record[tag_start..pos + tag_end].to_uppercase();
                pos += tag_end + 1;
                if let Some(close_start) = record[pos..].find(&format!("</{}>", tag_name)) {
                    let value = record[pos..pos + close_start].trim().to_string();
                    pos += close_start + tag_name.len() + 3;
                    match tag_name.as_str() {
                        "CALL" => call = value,
                        "FREQ" => freq = value.parse().unwrap_or(0.0),
                        "MODE" => mode = Mode::from_str(&value).unwrap_or(Mode::CW),
                        "QSO_DATE" => date_str = value,
                        "TIME_ON" => time_str = value,
                        "RST_SENT" => rst_sent = Some(value),
                        "RST_RCVD" => rst_rcvd = Some(value),
                        "BAND" => band = Band::from_name(&value),
                        "GRIDSQUARE" | "GRID" => grid = Some(value),
                        "QTH" => qth = Some(value),
                        "RIG" => rig = Some(value),
                        "NAME" => name = Some(value),
                        "COMMENT" => comment = Some(value),
                        _ => {}
                    }
                } else {
                    pos += 1;
                }
            } else {
                pos += 1;
            }
        }

        if call.is_empty() {
            continue;
        }
        let date_time_on = parse_adif_datetime(&date_str, &time_str);
        if band.is_none() && freq > 0.0 {
            band = Band::from_freq_mhz(freq);
        }
        let mut qso = Qso::new(call, freq, mode, date_time_on);
        qso.rst_sent = rst_sent;
        qso.rst_rcvd = rst_rcvd;
        qso.band = band;
        qso.grid = grid;
        qso.qth = qth;
        qso.rig = rig;
        qso.name = name;
        qso.comment = comment;
        qsos.push(qso);
    }
    qsos
}

/// Export QSOs to a standalone SQLite3 database file
pub fn qsos_to_sqlite(qsos: &[Qso], path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let conn = Connection::open(path)?;
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS qsos (
            id TEXT PRIMARY KEY,
            date_time_on TEXT NOT NULL,
            call TEXT NOT NULL,
            freq REAL NOT NULL,
            mode TEXT NOT NULL,
            rst_sent TEXT,
            rst_rcvd TEXT,
            date_time_off TEXT,
            qth TEXT,
            rig TEXT,
            address TEXT,
            age INTEGER,
            altitude INTEGER,
            band TEXT,
            email TEXT,
            lat REAL,
            lon REAL,
            my_altitude INTEGER,
            my_antenna TEXT,
            my_city TEXT,
            my_country TEXT,
            my_lat REAL,
            my_lon REAL,
            my_name TEXT,
            my_rig TEXT,
            name TEXT,
            operator TEXT,
            owner_callsign TEXT,
            qsl_r_date TEXT,
            qsl_s_date TEXT,
            qsl_rcvd TEXT,
            qsl_sent TEXT,
            qsl_rcvd_via TEXT,
            qsl_sent_via TEXT,
            tx_power INTEGER,
            rx_power INTEGER,
            qso_complete TEXT,
            grid TEXT,
            my_grid TEXT,
            comment TEXT
        );"
    )?;

    for qso in qsos {
        conn.execute(
            "INSERT OR REPLACE INTO qsos (id, date_time_on, call, freq, mode, rst_sent, rst_rcvd,
                date_time_off, qth, rig, address, age, altitude, band, email, lat, lon,
                my_altitude, my_antenna, my_city, my_country, my_lat, my_lon, my_name, my_rig,
                name, operator, owner_callsign, qsl_r_date, qsl_s_date, qsl_rcvd, qsl_sent,
                qsl_rcvd_via, qsl_sent_via, tx_power, rx_power, qso_complete, grid, my_grid, comment)
             VALUES (?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,?11,?12,?13,?14,?15,?16,?17,?18,?19,?20,
                ?21,?22,?23,?24,?25,?26,?27,?28,?29,?30,?31,?32,?33,?34,?35,?36,?37,?38,?39,?40)",
            params![
                qso.id,
                qso.date_time_on.to_rfc3339(),
                qso.call,
                qso.freq,
                qso.mode.as_str(),
                qso.rst_sent,
                qso.rst_rcvd,
                qso.date_time_off.map(|d| d.to_rfc3339()),
                qso.qth,
                qso.rig,
                qso.address,
                qso.age,
                qso.altitude,
                qso.band.map(|b| b.name()),
                qso.email,
                qso.lat,
                qso.lon,
                qso.my_altitude,
                qso.my_antenna,
                qso.my_city,
                qso.my_country,
                qso.my_lat,
                qso.my_lon,
                qso.my_name,
                qso.my_rig,
                qso.name,
                qso.operator,
                qso.owner_callsign,
                qso.qsl_r_date.map(|d| d.to_rfc3339()),
                qso.qsl_s_date.map(|d| d.to_rfc3339()),
                qso.qsl_rcvd.map(|v| format!("{:?}", v)),
                qso.qsl_sent.map(|v| format!("{:?}", v)),
                qso.qsl_rcvd_via.map(|v| format!("{:?}", v)),
                qso.qsl_sent_via.map(|v| format!("{:?}", v)),
                qso.tx_power,
                qso.rx_power,
                qso.qso_complete.map(|v| format!("{:?}", v)),
                qso.grid,
                qso.my_grid,
                qso.comment,
            ],
        )?;
    }
    Ok(())
}

/// Import QSOs from a standalone SQLite3 database file
pub fn sqlite_to_qsos(path: &str) -> Result<Vec<Qso>, Box<dyn std::error::Error>> {
    let conn = Connection::open(path)?;
    let mut stmt = conn.prepare("SELECT * FROM qsos ORDER BY date_time_on DESC")?;
    let mut rows = stmt.query([])?;
    let mut qsos = Vec::new();
    while let Some(row) = rows.next()? {
        qsos.push(sqlite_row_to_qso(row)?);
    }
    Ok(qsos)
}

fn sqlite_row_to_qso(row: &rusqlite::Row) -> rusqlite::Result<Qso> {
    let date_time_on_str: String = row.get("date_time_on")?;
    let date_time_on = chrono::DateTime::parse_from_rfc3339(&date_time_on_str)
        .map(|d| d.to_utc())
        .unwrap_or_else(|_| chrono::Utc::now());

    let date_time_off: Option<chrono::DateTime<chrono::Utc>> = row.get::<_, Option<String>>("date_time_off")?
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
        .map(|d| d.to_utc());

    let mode_str: String = row.get("mode")?;
    let mode = Mode::from_str(&mode_str).unwrap_or(Mode::CW);

    let band_str: Option<String> = row.get("band")?;
    let band = band_str.and_then(|s| Band::from_name(&s));

    let qsl_rcvd: Option<QslRcvd> = row.get::<_, Option<String>>("qsl_rcvd")?
        .and_then(|s| match s.as_str() {
            "Y" => Some(QslRcvd::Y),
            "N" => Some(QslRcvd::N),
            "R" => Some(QslRcvd::R),
            "I" => Some(QslRcvd::I),
            "V" => Some(QslRcvd::V),
            _ => None,
        });

    let qsl_sent: Option<QslSent> = row.get::<_, Option<String>>("qsl_sent")?
        .and_then(|s| match s.as_str() {
            "Y" => Some(QslSent::Y),
            "N" => Some(QslSent::N),
            "R" => Some(QslSent::R),
            "Q" => Some(QslSent::Q),
            "I" => Some(QslSent::I),
            _ => None,
        });

    let qsl_rcvd_via: Option<QslVia> = row.get::<_, Option<String>>("qsl_rcvd_via")?
        .and_then(|s| match s.as_str() {
            "B" => Some(QslVia::B),
            "D" => Some(QslVia::D),
            "E" => Some(QslVia::E),
            "M" => Some(QslVia::M),
            _ => None,
        });

    let qsl_sent_via: Option<QslVia> = row.get::<_, Option<String>>("qsl_sent_via")?
        .and_then(|s| match s.as_str() {
            "B" => Some(QslVia::B),
            "D" => Some(QslVia::D),
            "E" => Some(QslVia::E),
            "M" => Some(QslVia::M),
            _ => None,
        });

    let qso_complete: Option<QsoComplete> = row.get::<_, Option<String>>("qso_complete")?
        .and_then(|s| match s.as_str() {
            "Y" => Some(QsoComplete::Y),
            "N" => Some(QsoComplete::N),
            "NIL" => Some(QsoComplete::NIL),
            "UNCERTAIN" => Some(QsoComplete::UNCERTAIN),
            _ => None,
        });

    Ok(Qso {
        id: row.get("id")?,
        date_time_on,
        call: row.get("call")?,
        freq: row.get("freq")?,
        mode,
        rst_sent: row.get("rst_sent")?,
        rst_rcvd: row.get("rst_rcvd")?,
        date_time_off,
        qth: row.get("qth")?,
        rig: row.get("rig")?,
        address: row.get("address")?,
        age: row.get("age")?,
        altitude: row.get("altitude")?,
        band,
        email: row.get("email")?,
        lat: row.get("lat")?,
        lon: row.get("lon")?,
        my_altitude: row.get("my_altitude")?,
        my_antenna: row.get("my_antenna")?,
        my_city: row.get("my_city")?,
        my_country: row.get("my_country")?,
        my_lat: row.get("my_lat")?,
        my_lon: row.get("my_lon")?,
        my_name: row.get("my_name")?,
        my_rig: row.get("my_rig")?,
        name: row.get("name")?,
        operator: row.get("operator")?,
        owner_callsign: row.get("owner_callsign")?,
        qsl_r_date: row.get::<_, Option<String>>("qsl_r_date")?
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.to_utc()),
        qsl_s_date: row.get::<_, Option<String>>("qsl_s_date")?
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.to_utc()),
        qsl_rcvd,
        qsl_sent,
        qsl_rcvd_via,
        qsl_sent_via,
        tx_power: row.get("tx_power")?,
        rx_power: row.get("rx_power")?,
        qso_complete,
        grid: row.get("grid")?,
        my_grid: row.get("my_grid")?,
        comment: row.get("comment")?,
    })
}
