//! Physics versions. Every constant that differs between game versions goes through here.

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum PhysicsVersion {
    V22 = 0,
    /// Reserved: tables are filled in late in the project.
    V21 = 1,
}

impl PhysicsVersion {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::V22),
            1 => Some(Self::V21),
            _ => None,
        }
    }
}

/// Speed portal settings. See docs/01-physics-spec.md section 3.
#[derive(Clone, Copy, Debug)]
pub struct SpeedParams {
    pub player_speed: f64,
    pub speed_mult: f64,
    pub y_start: f64,
    pub gravity: f64,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Speed {
    Half = 0,
    Normal = 1,
    Double = 2,
    Triple = 3,
    Quad = 4,
}

impl Speed {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0 => Some(Self::Half),
            1 => Some(Self::Normal),
            2 => Some(Self::Double),
            3 => Some(Self::Triple),
            4 => Some(Self::Quad),
            _ => None,
        }
    }
}

/// Returns `None` for versions whose tables are not filled in yet.
pub fn speed_params(version: PhysicsVersion, speed: Speed) -> Option<SpeedParams> {
    match version {
        PhysicsVersion::V22 => Some(match speed {
            Speed::Half => SpeedParams { player_speed: 0.7, speed_mult: 5.98, y_start: 10.62, gravity: 0.940199 },
            Speed::Normal => SpeedParams { player_speed: 0.9, speed_mult: 5.77, y_start: 11.18, gravity: 0.958199 },
            Speed::Double => SpeedParams { player_speed: 1.1, speed_mult: 5.87, y_start: 11.42, gravity: 0.957199 },
            Speed::Triple => SpeedParams { player_speed: 1.3, speed_mult: 6.00, y_start: 11.23, gravity: 0.961199 },
            Speed::Quad => SpeedParams { player_speed: 1.6, speed_mult: 6.00, y_start: 11.23, gravity: 0.961199 },
        }),
        PhysicsVersion::V21 => None,
    }
}
