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
    use geoutils::Location;
    
    /// Calculate distance between two coordinates
    pub fn calc_distance(
        from_lat: f64,
        from_lon: f64,
        to_lat: f64,
        to_lon: f64,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let from = Location::new(from_lat, from_lon);
        let to = Location::new(to_lat, to_lon);
        Ok(from.distance_to(&to).unwrap().meters())
    }
}

#[cfg(test)]
mod tests {
    mod sunrise {

    }
    mod geo {
        use crate::calc::geo;
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
