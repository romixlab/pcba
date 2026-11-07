//! Every number the assembly drawing uses, with its default; a board may override any of them in a TOML file.

use serde::{Deserialize, Serialize};

/// Settings of `pcba fab`; lengths in millimetres on the board (the page scales them with the board)
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct FabSettings {
    /// Smallest reference height inside a body; below it the reference goes outside with a leader
    pub min_text: f64,
    /// Largest reference height inside a body
    pub max_text: f64,
    /// Share of the body's width and height a reference may fill
    pub text_fill: f64,
    /// Width of one character as a share of its height (the drawing uses a monospace font)
    pub char_width: f64,
    /// Height of a reference placed outside its body
    pub callout_text: f64,
    /// Gap between a part (or another label) and a reference placed outside
    pub callout_gap: f64,
    /// How far from its part a reference placed outside may go
    pub callout_reach: f64,
    /// Gap a reference placed outside keeps from the board outline line, holes, test point and fiducial symbols
    pub edge_gap: f64,
    /// Margin around the board on each page
    pub margin: f64,
    /// Line width of the board outline
    pub outline_width: f64,
    /// Line width of part bodies
    pub body_width: f64,
    /// Line width of leaders, DNP crosses and symbols
    pub thin_width: f64,
    /// Radius of the test point symbol
    pub testpoint_radius: f64,
    /// Size of a drill symbol on the drill page
    pub drill_symbol: f64,
    /// Footprint libraries (prefix of `lib:name`) that are graphics, not parts: logos, net ties, labels
    pub skip_libraries: Vec<String>,
}

impl Default for FabSettings {
    fn default() -> Self {
        FabSettings {
            min_text: 0.45,
            max_text: 1.6,
            text_fill: 0.8,
            char_width: 0.6,
            callout_text: 0.7,
            callout_gap: 0.25,
            callout_reach: 8.0,
            edge_gap: 0.15,
            margin: 6.0,
            outline_width: 0.2,
            body_width: 0.08,
            thin_width: 0.05,
            testpoint_radius: 0.5,
            drill_symbol: 0.9,
            skip_libraries: vec![
                "VHRD_Symbol:".into(),
                "VHRD_NetTie:".into(),
                "NetTie:".into(),
                "Symbol:".into(),
                "kibuzzard".into(),
            ],
        }
    }
}

impl FabSettings {
    /// Settings from a TOML file's text; missing fields keep their defaults
    pub fn from_toml(text: &str) -> Result<Self, String> {
        toml::from_str(text).map_err(|e| e.to_string())
    }
}
