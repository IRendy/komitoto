use rusqlite::{params, Connection, Result as SqlResult};
use chrono::{DateTime, Utc};
use komitoto_types::*;

const DB_PATH: &str = "komitoto.db";

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(path: Option<&str>) -> SqlResult<Self> {
        let conn = Connection::open(path.unwrap_or(DB_PATH))?;
        let db = Database { conn };
        db.init()?;
        Ok(db)
    }

    fn init(&self) -> SqlResult<()> {
        self.conn.execute_batch(
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
        Ok(())
    }

    pub fn add_qso(&self, qso: &Qso) -> SqlResult<()> {
        self.conn.execute(
            "INSERT INTO qsos (id, date_time_on, call, freq, mode, rst_sent, rst_rcvd,
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
        Ok(())
    }

    pub fn list_qsos(&self, limit: Option<u32>) -> SqlResult<Vec<Qso>> {
        let sql = match limit {
            Some(_) => "SELECT * FROM qsos ORDER BY date_time_on DESC LIMIT ?1",
            None => "SELECT * FROM qsos ORDER BY date_time_on DESC",
        };
        let mut stmt = self.conn.prepare(sql)?;
        let rows = if let Some(lim) = limit {
            stmt.query(params![lim])?
        } else {
            stmt.query([])?
        };
        Self::rows_to_qsos(rows)
    }

    pub fn get_qso(&self, id: &str) -> SqlResult<Option<Qso>> {
        let mut stmt = self.conn.prepare("SELECT * FROM qsos WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        match rows.next()? {
            Some(row) => Ok(Some(Self::row_to_qso(row)?)),
            None => Ok(None),
        }
    }

    pub fn delete_qso(&self, id: &str) -> SqlResult<bool> {
        let affected = self.conn.execute("DELETE FROM qsos WHERE id = ?1", params![id])?;
        Ok(affected > 0)
    }

    pub fn update_qso(&self, qso: &Qso) -> SqlResult<bool> {
        let affected = self.conn.execute(
            "UPDATE qsos SET
                date_time_on = ?2, call = ?3, freq = ?4, mode = ?5,
                rst_sent = ?6, rst_rcvd = ?7, date_time_off = ?8, qth = ?9,
                rig = ?10, address = ?11, age = ?12, altitude = ?13, band = ?14,
                email = ?15, lat = ?16, lon = ?17, my_altitude = ?18, my_antenna = ?19,
                my_city = ?20, my_country = ?21, my_lat = ?22, my_lon = ?23,
                my_name = ?24, my_rig = ?25, name = ?26, operator = ?27,
                owner_callsign = ?28, qsl_r_date = ?29, qsl_s_date = ?30,
                qsl_rcvd = ?31, qsl_sent = ?32, qsl_rcvd_via = ?33, qsl_sent_via = ?34,
                tx_power = ?35, rx_power = ?36, qso_complete = ?37, grid = ?38,
                my_grid = ?39, comment = ?40
             WHERE id = ?1",
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
        Ok(affected > 0)
    }

    pub fn search_qsos(&self, call_pattern: &str) -> SqlResult<Vec<Qso>> {
        let pattern = call_pattern.replace('*', "%");
        let mut stmt = self.conn.prepare(
            "SELECT * FROM qsos WHERE call LIKE ?1 ORDER BY date_time_on DESC"
        )?;
        let rows = stmt.query(params![format!("%{}%", pattern)])?;
        Self::rows_to_qsos(rows)
    }

    pub fn get_all_qsos(&self) -> SqlResult<Vec<Qso>> {
        self.list_qsos(None)
    }

    fn row_to_qso(row: &rusqlite::Row) -> SqlResult<Qso> {
        let date_time_on_str: String = row.get("date_time_on")?;
        let date_time_on = DateTime::parse_from_rfc3339(&date_time_on_str)
            .map(|d| d.to_utc())
            .unwrap_or_else(|_| Utc::now());

        let date_time_off: Option<DateTime<Utc>> = row.get::<_, Option<String>>("date_time_off")?
            .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|d| d.to_utc());

        let mode_str: String = row.get("mode")?;
        let mode = Mode::from_str(&mode_str).unwrap_or(Mode::CW);

        let band_str: Option<String> = row.get("band")?;
        let band = band_str.and_then(|s| Band::from_name(&s));

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
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.to_utc()),
            qsl_s_date: row.get::<_, Option<String>>("qsl_s_date")?
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|d| d.to_utc()),
            qsl_rcvd: row.get::<_, Option<String>>("qsl_rcvd")?
                .and_then(|s| match s.as_str() {
                    "Y" => Some(QslRcvd::Y),
                    "N" => Some(QslRcvd::N),
                    "R" => Some(QslRcvd::R),
                    "I" => Some(QslRcvd::I),
                    "V" => Some(QslRcvd::V),
                    _ => None,
                }),
            qsl_sent: row.get::<_, Option<String>>("qsl_sent")?
                .and_then(|s| match s.as_str() {
                    "Y" => Some(QslSent::Y),
                    "N" => Some(QslSent::N),
                    "R" => Some(QslSent::R),
                    "Q" => Some(QslSent::Q),
                    "I" => Some(QslSent::I),
                    _ => None,
                }),
            qsl_rcvd_via: row.get::<_, Option<String>>("qsl_rcvd_via")?
                .and_then(|s| match s.as_str() {
                    "B" => Some(QslVia::B),
                    "D" => Some(QslVia::D),
                    "E" => Some(QslVia::E),
                    "M" => Some(QslVia::M),
                    _ => None,
                }),
            qsl_sent_via: row.get::<_, Option<String>>("qsl_sent_via")?
                .and_then(|s| match s.as_str() {
                    "B" => Some(QslVia::B),
                    "D" => Some(QslVia::D),
                    "E" => Some(QslVia::E),
                    "M" => Some(QslVia::M),
                    _ => None,
                }),
            tx_power: row.get("tx_power")?,
            rx_power: row.get("rx_power")?,
            qso_complete: row.get::<_, Option<String>>("qso_complete")?
                .and_then(|s| match s.as_str() {
                    "Y" => Some(QsoComplete::Y),
                    "N" => Some(QsoComplete::N),
                    "NIL" => Some(QsoComplete::NIL),
                    "UNCERTAIN" => Some(QsoComplete::UNCERTAIN),
                    _ => None,
                }),
            grid: row.get("grid")?,
            my_grid: row.get("my_grid")?,
            comment: row.get("comment")?,
        })
    }

    fn rows_to_qsos(mut rows: rusqlite::Rows) -> SqlResult<Vec<Qso>> {
        let mut qsos = Vec::new();
        while let Some(row) = rows.next()? {
            qsos.push(Self::row_to_qso(row)?);
        }
        Ok(qsos)
    }
}
