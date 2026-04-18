//! Common types used across all PDF Office crates.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// RGBA color (values 0–255)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0, a: 255 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255, a: 255 };
    pub const TRANSPARENT: Color = Color { r: 0, g: 0, b: 0, a: 0 };

    pub fn from_hex(hex: &str) -> Option<Self> {
        let hex = hex.trim_start_matches('#');
        let parse = |s: &str| u8::from_str_radix(s, 16).ok();
        match hex.len() {
            6 => Some(Color {
                r: parse(&hex[0..2])?,
                g: parse(&hex[2..4])?,
                b: parse(&hex[4..6])?,
                a: 255,
            }),
            8 => Some(Color {
                r: parse(&hex[0..2])?,
                g: parse(&hex[2..4])?,
                b: parse(&hex[4..6])?,
                a: parse(&hex[6..8])?,
            }),
            _ => None,
        }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}{:02X}", self.r, self.g, self.b, self.a)
    }

    pub fn to_css(&self) -> String {
        format!("rgba({},{},{},{})", self.r, self.g, self.b, self.a as f32 / 255.0)
    }
}

/// Font weight
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Normal,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    Numeric(u16),
}

impl FontWeight {
    pub fn as_number(&self) -> u16 {
        match self {
            FontWeight::Thin => 100,
            FontWeight::ExtraLight => 200,
            FontWeight::Light => 300,
            FontWeight::Normal => 400,
            FontWeight::Medium => 500,
            FontWeight::SemiBold => 600,
            FontWeight::Bold => 700,
            FontWeight::ExtraBold => 800,
            FontWeight::Black => 900,
            FontWeight::Numeric(n) => *n,
        }
    }
}

impl Default for FontWeight {
    fn default() -> Self { FontWeight::Normal }
}

/// Font style
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

/// Font specification
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FontSpec {
    pub family: String,
    pub size: f32,        // pt
    pub weight: FontWeight,
    pub style: FontStyle,
}

impl Default for FontSpec {
    fn default() -> Self {
        FontSpec {
            family: "Liberation Serif".into(),
            size: 12.0,
            weight: FontWeight::Normal,
            style: FontStyle::Normal,
        }
    }
}

/// 2D point
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

/// 2D rectangle
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    pub fn contains(&self, p: &Point) -> bool {
        p.x >= self.x && p.x <= self.x + self.width
            && p.y >= self.y && p.y <= self.y + self.height
    }
}

/// 2D transform (position + size + rotation)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,  // degrees
}

impl Default for Transform {
    fn default() -> Self {
        Transform { x: 0.0, y: 0.0, width: 100.0, height: 100.0, rotation: 0.0 }
    }
}

/// Named style reference
pub type StyleRef = String;

/// Generic attributes map (for extensible properties)
pub type Attrs = HashMap<String, serde_json::Value>;

/// Page size and margins
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageLayout {
    pub width: f32,        // mm
    pub height: f32,       // mm
    pub margin_top: f32,   // mm
    pub margin_bottom: f32,
    pub margin_left: f32,
    pub margin_right: f32,
    pub orientation: PageOrientation,
}

impl Default for PageLayout {
    fn default() -> Self {
        PageLayout {
            width: 210.0,
            height: 297.0,
            margin_top: 25.4,
            margin_bottom: 25.4,
            margin_left: 25.4,
            margin_right: 25.4,
            orientation: PageOrientation::Portrait,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum PageOrientation {
    #[default]
    Portrait,
    Landscape,
}

/// Border style
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Border {
    pub style: BorderStyle,
    pub width: f32,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum BorderStyle {
    #[default]
    None,
    Solid,
    Dashed,
    Dotted,
    Double,
}

/// Text alignment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum TextAlign {
    #[default]
    Left,
    Right,
    Center,
    Justify,
}

/// Vertical alignment
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum VerticalAlign {
    #[default]
    Top,
    Middle,
    Bottom,
}
