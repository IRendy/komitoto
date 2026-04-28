use std::error::Error;
use komitoto_db::Database;
use komitoto_types::*;

/// QSO Logbook Service - handles all logbook business logic
pub struct QsoService {
    db: Database,
}

impl QsoService {
    pub fn new(db_path: Option<&str>) -> Result<Self, Box<dyn Error>> {
        // Check for KOMITOTO_LOGBOOK environment variable first
        let db_path_str: Option<String> = if let Some(path) = std::env::var("KOMITOTO_LOGBOOK").ok() {
            Some(path)  // Keep as Option<String>
        } else {
            db_path.map(|s| s.to_string())
        };
        
        Ok(Self {
            db: Database::open(db_path_str.as_deref())?,
        })
    }

    /// Add a new QSO record
    pub fn add_qso(&self, qso: &Qso) -> Result<String, Box<dyn Error>> {
        self.db.add_qso(qso)?;
        Ok(qso.id.clone())
    }

    /// List QSO records with optional limit
    pub fn list_qsos(&self, limit: Option<u32>) -> Result<Vec<Qso>, Box<dyn Error>> {
        Ok(self.db.list_qsos(limit)?)
    }

    /// Get a specific QSO by ID
    pub fn get_qso(&self, id: &str) -> Result<Option<Qso>, Box<dyn Error>> {
        Ok(self.db.get_qso(id)?)
    }

    /// Update an existing QSO
    pub fn update_qso(&self, qso: &Qso) -> Result<bool, Box<dyn Error>> {
        Ok(self.db.update_qso(qso)?)
    }

    /// Delete a QSO by ID
    pub fn delete_qso(&self, id: &str) -> Result<bool, Box<dyn Error>> {
        Ok(self.db.delete_qso(id)?)
    }

    /// Search QSOs by callsign pattern (supports wildcard *)
    pub fn search_qsos(&self, pattern: &str) -> Result<Vec<Qso>, Box<dyn Error>> {
        Ok(self.db.search_qsos(pattern)?)
    }

    /// Get all QSOs
    pub fn get_all_qsos(&self) -> Result<Vec<Qso>, Box<dyn Error>> {
        Ok(self.db.get_all_qsos()?)
    }

    #[allow(dead_code)]
    /// Parse frequency and automatically detect band
    pub fn parse_freq_with_band(freq: f64) -> (f64, Option<Band>) {
        let band = Band::from_freq_mhz(freq);
        (freq, band)
    }

    #[allow(dead_code)]
    /// Validate mode string
    pub fn validate_mode(mode_str: &str) -> Result<Mode, String> {
        Mode::from_str(mode_str).ok_or_else(|| format!("Unknown mode: {}", mode_str))
    }

    #[allow(dead_code)]
    /// Parse RST string to integer (for numeric RST values like 59, 57)
    pub fn parse_rst(rst_str: &str) -> Option<i32> {
        rst_str.parse::<i32>().ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rst() {
        assert_eq!(QsoService::parse_rst("59"), Some(59));
        assert_eq!(QsoService::parse_rst("57"), Some(57));
        assert_eq!(QsoService::parse_rst("abc"), None);
    }

    #[test]
    fn test_validate_mode() {
        assert!(QsoService::validate_mode("FM").is_ok());
        assert!(QsoService::validate_mode("cw").is_ok());
        assert!(QsoService::validate_mode("INVALID").is_err());
    }
}
