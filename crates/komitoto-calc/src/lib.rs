pub mod sunrise {
    use chrono::{DateTime, NaiveDate, Utc};
    use sunrise::{Coordinates, DawnType, SolarDay, SolarEvent};

    #[derive(Debug, Clone)]
    pub struct SunTimes {
        pub sunrise: DateTime<Utc>,
        pub sunset: DateTime<Utc>,
        pub dawn: DateTime<Utc>,
        pub dusk: DateTime<Utc>,
    }

    pub fn calc_sunrise(
        date: NaiveDate,
        lat: f64,
        lon: f64,
        altitude: f64,
        dawn_type: Option<DawnType>,
    ) -> Result<SunTimes, Box<dyn std::error::Error>> {
        let coord = Coordinates::new(lat, lon).unwrap();
        let occasion = SolarDay::new(coord, date).with_altitude(altitude);

        let sunrise = occasion.event_time(SolarEvent::Sunrise).unwrap();
        let sunset = occasion.event_time(SolarEvent::Sunset).unwrap();

        let actual_dawn_type = dawn_type.unwrap_or(DawnType::Civil);
        let dawn = occasion.event_time(SolarEvent::Dawn(actual_dawn_type)).unwrap();
        let dusk = occasion.event_time(SolarEvent::Dusk(actual_dawn_type)).unwrap();

        Ok(SunTimes {
            sunrise,
            sunset,
            dawn,
            dusk,
        })
    }
}

pub mod geo {
    use geo::{Contains, Coord, Point, Polygon};
    use lazy_static::lazy_static;
    use serde_json::Value as JsonValue;

    const CQ_GEOJSON: &str = include_str!("../../../hamradio-zones-geojson-main/cqzones.geojson");
    const ITU_GEOJSON: &str = include_str!("../../../hamradio-zones-geojson-main/ituzones.geojson");

    /// Calculate distance between two coordinates using Vincenty formula
    pub fn calc_distance(
        from_lat: f64,
        from_lon: f64,
        to_lat: f64,
        to_lon: f64,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let from = Point::<f64>::new(from_lon, from_lat);
        let to = Point::<f64>::new(to_lon, to_lat);
        
        use geo::algorithm::vincenty_distance::VincentyDistance;
        Ok(VincentyDistance::vincenty_distance(&from, &to)?)
    }

    #[derive(Debug)]
    pub struct ZoneFinder {
        cq_zones: Vec<(Polygon<f64>, i32)>,
        ituzones: Vec<(Polygon<f64>, i32)>,
    }

    impl ZoneFinder {
        fn load_zones(json_str: &str, zone_key: &str) -> Result<Vec<(Polygon<f64>, i32)>, Box<dyn std::error::Error>> {
            let mut zones = Vec::new();
            let data: JsonValue = serde_json::from_str(json_str)?;
            
            if let Some(features) = data.get("features").and_then(|v| v.as_array()) {
                for feature in features {
                    if let Some(props) = feature.get("properties") {
                        if let Some(num) = props.get(zone_key).and_then(|v| v.as_i64()) {
                            let zone_num = num as i32;
                            
                            if let Some(geom) = feature.get("geometry") {
                                if let Some(poly) = Self::extract_polygon_from_geom(geom)? {
                                    zones.push((poly, zone_num));
                                }
                            }
                        }
                    }
                }
            }

            Ok(zones)
        }

        pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
            let cq_zones = Self::load_zones(CQ_GEOJSON, "cq_zone_number")?;
            let ituzones = Self::load_zones(ITU_GEOJSON, "itu_zone_number")?;

            Ok(Self { 
                cq_zones,
                ituzones 
            })
        }

        fn extract_polygon_from_geom(geom: &JsonValue) -> Result<Option<Polygon<f64>>, Box<dyn std::error::Error>> {
            if geom["type"] == "Polygon" {
                let coords = geom["coordinates"][0].as_array()
                    .ok_or("Invalid polygon coordinates")?;
                
                let mut point_coords: Vec<(f64, f64)> = coords.iter()
                    .filter_map(|c| {
                        let arr = c.as_array()?;
                        if arr.len() >= 2 {
                            let lon = arr[0].as_f64()?;
                            let lat = arr[1].as_f64()?;
                            Some((lon, lat))
                        } else {
                            None
                        }
                    })
                    .collect();
                
                if point_coords.len() > 2 {
                    // Close the ring by adding first coordinate at the end
                    if let Some(first) = point_coords.get(0).cloned() {
                        point_coords.push(first);
                    }
                    
                    use geo::LineString;
                    let exterior = LineString(point_coords.into_iter().map(|(x, y)| Coord { x, y }).collect());
                    return Ok(Some(Polygon::new(exterior, vec![])));
                }
            }
            Ok(None)
        }

        pub fn find_zone(&self, lat: f64, lon: f64) -> Option<(String, i32, String, i32)> {
            let point = Point::<f64>::new(lon, lat);
            
            // Find CQ zone
            let mut cq_zone = None;
            for (poly, num) in &self.cq_zones {
                if poly.contains(&point) {
                    cq_zone = Some(("CQ".to_string(), *num));
                    break;
                }
            }

            // Find ITU zone
            let mut itu_zone = None;
            for (poly, num) in &self.ituzones {
                if poly.contains(&point) {
                    itu_zone = Some(("ITU".to_string(), *num));
                    break;
                }
            }

            cq_zone.and_then(|cq| {
                itu_zone.map(|itu| (cq.0, cq.1, itu.0, itu.1))
            })
        }
    }

    lazy_static! {
        static ref ZONE_FINDER: ZoneFinder = ZoneFinder::new().expect("Failed to create zone finder");
    }

    pub fn get_zone_finder() -> &'static ZoneFinder {
        &ZONE_FINDER
    }
}

pub mod maidenhead {
    /// Convert latitude/longitude to Maidenhead grid locator
    pub fn to_maidenhead(lat: f64, lon: f64, precision: usize) -> Result<String, String> {
        if !(2..=10).contains(&precision) || precision % 2 != 0 {
            return Err("Precision must be even number between 2 and 10".into());
        }
        if !(-90.0..=90.0).contains(&lat) {
            return Err("Latitude must be between -90 and 90".into());
        }
        if !(-180.0..=180.0).contains(&lon) {
            return Err("Longitude must be between -180 and 180".into());
        }

        let mut lon = lon + 180.0;  // 0-360
        let mut lat = lat + 90.0;   // 0-180
        
        let mut grid = String::new();

        // Field (2 chars): 20 deg x 10 deg
        let field_lon = (lon / 20.0).floor() as u8;
        let field_lat = (lat / 10.0).floor() as u8;
        grid.push((b'A' + field_lon) as char);
        grid.push((b'A' + field_lat) as char);
        
        lon %= 20.0;
        lat %= 10.0;

        if precision >= 4 {
            // Square (2 chars): 2 deg x 1 deg
            let square_lon = (lon / 2.0).floor() as u8;
            let square_lat = lat.floor() as u8;
            grid.push((b'0' + square_lon) as char);
            grid.push((b'0' + square_lat) as char);
            
            lon %= 2.0;
            lat %= 1.0;
        }

        if precision >= 6 {
            // Subsquare (2 chars): 5' x 2.5'
            let lon_min = lon * 60.0;  // convert to minutes
            let lat_min = lat * 60.0;
            let subsq_lon = (lon_min / 5.0).floor() as u8;
            let subsq_lat = (lat_min / 2.5).floor() as u8;
            grid.push((b'a' + subsq_lon) as char);
            grid.push((b'a' + subsq_lat) as char);
            
            lon = lon_min % 5.0;
            lat = lat_min % 2.5;
        }

        if precision >= 8 {
            // Extended square (2 chars): ~385m
            let ext_lon = (lon * 24.0 / 5.0).floor() as u8;
            let ext_lat = (lat * 24.0 / 2.5).floor() as u8;
            grid.push((b'0' + ext_lon) as char);
            grid.push((b'0' + ext_lat) as char);
        }

        if precision >= 10 {
            // Further extended
            let lon_sec = lon * 24.0 * 60.0 / 5.0;
            let lat_sec = lat * 24.0 * 60.0 / 2.5;
            let ext2_lon = (lon_sec / 24.0).floor() as u8 % 24;
            let ext2_lat = (lat_sec / 24.0).floor() as u8 % 24;
            grid.push((b'a' + ext2_lon) as char);
            grid.push((b'a' + ext2_lat) as char);
        }

        Ok(grid)
    }

    /// Convert Maidenhead grid locator to latitude/longitude (center of cell)
    pub fn from_maidenhead(grid: &str) -> Result<(f64, f64), String> {
        let g: String = grid.chars()
            .filter(|c| !c.is_whitespace())
            .collect();
        
        if g.len() < 4 || g.len() > 10 || g.len() % 2 != 0 {
            return Err("Grid must be 4-10 characters, even length".into());
        }
        
        let chars: Vec<char> = g.chars().collect();
        let mut lon = 0.0f64;
        let mut lat = 0.0f64;
        let mut remaining_len = g.len();

        // Field
        let f_lon = chars[0].to_ascii_uppercase() as u8 - b'A';
        let f_lat = chars[1].to_ascii_uppercase() as u8 - b'A';
        if f_lon > 17 || f_lat > 17 {
            return Err("Invalid field characters (must be A-R)".into());
        }
        lon += f_lon as f64 * 20.0;
        lat += f_lat as f64 * 10.0;
        remaining_len -= 2;

        // Square
        let s_lon = chars[2] as u8 - b'0';
        let s_lat = chars[3] as u8 - b'0';
        if s_lon > 9 || s_lat > 9 {
            return Err("Invalid square characters (must be 0-9)".into());
        }
        lon += s_lon as f64 * 2.0;
        lat += s_lat as f64 * 1.0;
        remaining_len -= 2;

        if remaining_len >= 2 {
            let ss_lon = chars[4].to_ascii_lowercase() as u8 - b'a';
            let ss_lat = chars[5].to_ascii_lowercase() as u8 - b'a';
            if ss_lon > 23 || ss_lat > 23 {
                return Err("Invalid subsquare characters (must be a-x)".into());
            }
            lon += ss_lon as f64 * 5.0 / 60.0;
            lat += ss_lat as f64 * 2.5 / 60.0;
            remaining_len -= 2;
        }

        if remaining_len >= 2 {
            let e_lon = chars[6] as u8 - b'0';
            let e_lat = chars[7] as u8 - b'0';
            if e_lon > 23 || e_lat > 23 {
                return Err("Invalid extended characters".into());
            }
            lon += e_lon as f64 * 5.0 / 60.0 / 24.0;
            lat += e_lat as f64 * 2.5 / 60.0 / 24.0;
        }

        // Return center of cell
        let cell_w = cell_width(g.len());
        let cell_h = cell_height(g.len());
        
        Ok((
            lat - 90.0 + cell_h / 2.0,
            lon - 180.0 + cell_w / 2.0,
        ))
    }

    fn cell_width(precision: usize) -> f64 {
        match precision {
            2 => 20.0,
            4 => 2.0,
            6 => 5.0 / 60.0,
            8 => 5.0 / 60.0 / 24.0,
            10 => 5.0 / 60.0 / 24.0 / 24.0,
            _ => 0.0,
        }
    }

    fn cell_height(precision: usize) -> f64 {
        match precision {
            2 => 10.0,
            4 => 1.0,
            6 => 2.5 / 60.0,
            8 => 2.5 / 60.0 / 24.0,
            10 => 2.5 / 60.0 / 24.0 / 24.0,
            _ => 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    mod sunrise {

    }
    mod geo {
        use crate::geo;
        #[test]
        fn distance_test() {
            let beijing = (39.9042, 116.4074);
            let guangzhou = (23.1291, 113.2644);
            let shanghai = (31.2304, 121.4737);
            let hangzhou = (30.2428, 120.1500);
            let chengdu = (30.6799, 104.0679);

            let dis_bj_gz = geo::calc_distance(beijing.0, beijing.1, guangzhou.0, guangzhou.1).unwrap() / 1000.0;
            let dis_sh_hz = geo::calc_distance(shanghai.0, shanghai.1, hangzhou.0, hangzhou.1).unwrap() / 1000.0;
            let dis_sh_cd = geo::calc_distance(shanghai.0, shanghai.1, chengdu.0, chengdu.1).unwrap() / 1000.0;

            let bias = |a: f64, b: f64| {(a - b).abs()  < 10.0}; // error control

            assert!(bias(1888.59, dis_bj_gz));
            assert!(bias(159.5, dis_sh_hz));
            assert!(bias(1660.0, dis_sh_cd));
            }
        }
}
