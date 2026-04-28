/// SSTV encoding mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SstvMode {
    MartinM1,
    MartinM2,
    ScottieS1,
    ScottieS2,
    Robot36,
    Robot72,
    Pd50,
    Pd90,
    Pd120,
    Pd160,
    Pd180,
    Pd240,
    Pd290,
    Avt90,
}

impl SstvMode {
    /// Human-readable name for display.
    pub fn name(&self) -> &'static str {
        match self {
            Self::MartinM1 => "Martin M1",
            Self::MartinM2 => "Martin M2",
            Self::ScottieS1 => "Scottie S1",
            Self::ScottieS2 => "Scottie S2",
            Self::Robot36 => "Robot 36",
            Self::Robot72 => "Robot 72",
            Self::Pd50 => "PD 50",
            Self::Pd90 => "PD 90",
            Self::Pd120 => "PD 120",
            Self::Pd160 => "PD 160",
            Self::Pd180 => "PD 180",
            Self::Pd240 => "PD 240",
            Self::Pd290 => "PD 290",
            Self::Avt90 => "AVT 90",
        }
    }

    /// Parse from a CLI-style string (case-insensitive).
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace(['-', '_'], "").as_str() {
            "martinm1" | "m1" => Some(Self::MartinM1),
            "martinm2" | "m2" => Some(Self::MartinM2),
            "scotties1" | "s1" => Some(Self::ScottieS1),
            "scotties2" | "s2" => Some(Self::ScottieS2),
            "robot36" | "r36" => Some(Self::Robot36),
            "robot72" | "r72" => Some(Self::Robot72),
            "pd50" => Some(Self::Pd50),
            "pd90" => Some(Self::Pd90),
            "pd120" => Some(Self::Pd120),
            "pd160" => Some(Self::Pd160),
            "pd180" => Some(Self::Pd180),
            "pd240" => Some(Self::Pd240),
            "pd290" => Some(Self::Pd290),
            "avt90" => Some(Self::Avt90),
            _ => None,
        }
    }

    /// All supported SSTV modes.
    pub fn all() -> &'static [Self] {
        &[
            Self::MartinM1,
            Self::MartinM2,
            Self::ScottieS1,
            Self::ScottieS2,
            Self::Robot36,
            Self::Robot72,
            Self::Pd50,
            Self::Pd90,
            Self::Pd120,
            Self::Pd160,
            Self::Pd180,
            Self::Pd240,
            Self::Pd290,
            Self::Avt90,
        ]
    }

    /// Native resolution (width, height) for this mode.
    pub fn resolution(&self) -> (u32, u32) {
        match self {
            Self::Robot36 | Self::Robot72 => (320, 240),
            Self::Pd120 | Self::Pd180 | Self::Pd240 => (640, 496),
            Self::Pd160 => (512, 400),
            Self::Pd290 => (800, 616),
            _ => (320, 256),
        }
    }
}
