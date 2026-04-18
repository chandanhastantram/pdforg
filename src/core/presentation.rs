//! Presentation model — slides, elements, themes, animations.

use super::common::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A full presentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Presentation {
    pub id: Uuid,
    pub title: String,
    pub slides: Vec<Slide>,
    pub theme: Theme,
    pub slide_width: f32,   // pt (default 720 = 10 inches)
    pub slide_height: f32,  // pt (default 540 = 7.5 inches)
    pub notes_master: Option<NotesMaster>,
}

impl Default for Presentation {
    fn default() -> Self {
        Presentation {
            id: Uuid::new_v4(),
            title: "Untitled Presentation".into(),
            slides: vec![Slide::default()],
            theme: Theme::default(),
            slide_width: 720.0,
            slide_height: 540.0,
            notes_master: None,
        }
    }
}

/// A single slide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slide {
    pub id: Uuid,
    pub elements: Vec<SlideElement>,
    pub background: Background,
    pub animations: Vec<Animation>,
    pub transition: Option<Transition>,
    pub notes: String,
    pub hidden: bool,
    pub layout_ref: Option<String>,
}

impl Default for Slide {
    fn default() -> Self {
        Slide {
            id: Uuid::new_v4(),
            elements: vec![],
            background: Background::default(),
            animations: vec![],
            transition: None,
            notes: String::new(),
            hidden: false,
            layout_ref: None,
        }
    }
}

/// Slide background
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Background {
    Solid(Color),
    Gradient(Gradient),
    Image { data: Vec<u8>, mime_type: String, fit: ImageFit },
    Theme(String),
}

impl Default for Background {
    fn default() -> Self {
        Background::Solid(Color::WHITE)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Gradient {
    pub stops: Vec<GradientStop>,
    pub angle: f32,  // degrees
    pub kind: GradientKind,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradientStop {
    pub color: Color,
    pub position: f32,  // 0.0 – 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum GradientKind {
    #[default]
    Linear,
    Radial,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum ImageFit {
    #[default]
    Cover,
    Contain,
    Fill,
    None,
}

/// Slide elements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlideElement {
    TextBox(TextBoxEl),
    Image(ImageEl),
    Shape(ShapeEl),
    Table(TableEl),
    Chart(ChartEl),
    Group(GroupEl),
    Video(VideoEl),
}

impl SlideElement {
    pub fn id(&self) -> Uuid {
        match self {
            SlideElement::TextBox(e) => e.id,
            SlideElement::Image(e) => e.id,
            SlideElement::Shape(e) => e.id,
            SlideElement::Table(e) => e.id,
            SlideElement::Chart(e) => e.id,
            SlideElement::Group(e) => e.id,
            SlideElement::Video(e) => e.id,
        }
    }

    pub fn transform(&self) -> &Transform {
        match self {
            SlideElement::TextBox(e) => &e.transform,
            SlideElement::Image(e) => &e.transform,
            SlideElement::Shape(e) => &e.transform,
            SlideElement::Table(e) => &e.transform,
            SlideElement::Chart(e) => &e.transform,
            SlideElement::Group(e) => &e.transform,
            SlideElement::Video(e) => &e.transform,
        }
    }
}

/// Text box element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextBoxEl {
    pub id: Uuid,
    pub transform: Transform,
    pub paragraphs: Vec<SlideParagraph>,
    pub fill: Option<Color>,
    pub border: Option<Border>,
    pub padding: Padding,
    pub vertical_align: VerticalAlign,
    pub text_direction: TextDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideParagraph {
    pub runs: Vec<SlideRun>,
    pub align: TextAlign,
    pub space_before: f32,
    pub space_after: f32,
    pub line_height: f32,
    pub level: u8,
    pub bullet: Option<BulletStyle>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideRun {
    pub text: String,
    pub font: Option<FontSpec>,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub color: Option<Color>,
    pub link: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BulletStyle {
    Auto,
    None,
    Char(char),
    Image(Vec<u8>),
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Padding {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum TextDirection {
    #[default]
    LeftToRight,
    RightToLeft,
    TopToBottom,
    BottomToTop,
}

/// Image element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageEl {
    pub id: Uuid,
    pub transform: Transform,
    pub data: Vec<u8>,
    pub mime_type: String,
    pub alt: String,
    pub crop: Option<CropRect>,
    pub brightness: f32,
    pub contrast: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CropRect {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}

/// Shape element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeEl {
    pub id: Uuid,
    pub transform: Transform,
    pub kind: ShapeKind,
    pub fill: ShapeFill,
    pub stroke: Option<ShapeStroke>,
    pub shadow: Option<Shadow>,
    pub text: Option<TextBoxEl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeKind {
    Rectangle,
    RoundedRectangle { radius: f32 },
    Ellipse,
    Triangle,
    RightTriangle,
    Pentagon,
    Hexagon,
    Star { points: u8 },
    Arrow { direction: Direction },
    Line,
    Custom(String),  // SVG path
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ShapeFill {
    None,
    Solid(Color),
    Gradient(Gradient),
    Pattern,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShapeStroke {
    pub color: Color,
    pub width: f32,
    pub style: StrokeStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum StrokeStyle {
    #[default]
    Solid,
    Dashed,
    Dotted,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shadow {
    pub color: Color,
    pub offset_x: f32,
    pub offset_y: f32,
    pub blur: f32,
    pub spread: f32,
}

/// Table element on slide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableEl {
    pub id: Uuid,
    pub transform: Transform,
    pub rows: Vec<SlideTableRow>,
    pub col_widths: Vec<f32>,
    pub row_heights: Vec<f32>,
    pub style: SlideTableStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideTableRow {
    pub cells: Vec<SlideTableCell>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlideTableCell {
    pub paragraphs: Vec<SlideParagraph>,
    pub fill: Option<Color>,
    pub border: Option<Border>,
    pub vertical_align: VerticalAlign,
    pub rowspan: u32,
    pub colspan: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SlideTableStyle {
    pub first_row: bool,
    pub last_row: bool,
    pub banded_rows: bool,
    pub first_col: bool,
    pub last_col: bool,
}

/// Chart element on slide
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartEl {
    pub id: Uuid,
    pub transform: Transform,
    pub chart_data: Vec<u8>,  // serialized chart model
}

/// Group element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupEl {
    pub id: Uuid,
    pub transform: Transform,
    pub children: Vec<SlideElement>,
}

/// Video element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoEl {
    pub id: Uuid,
    pub transform: Transform,
    pub src: String,
    pub autoplay: bool,
    pub loop_video: bool,
    pub thumbnail: Option<Vec<u8>>,
}

/// Animation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    pub id: Uuid,
    pub element_id: Uuid,
    pub kind: AnimKind,
    pub trigger: AnimTrigger,
    pub delay_ms: u32,
    pub duration_ms: u32,
    pub easing: Easing,
    pub repeat: AnimRepeat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimKind {
    FadeIn,
    FadeOut,
    FlyIn(Direction),
    FlyOut(Direction),
    ZoomIn,
    ZoomOut,
    Pulse,
    Spin { turns: f32 },
    Bounce,
    Shake,
    Wipe(Direction),
    MotionPath(Vec<Point>),
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnimTrigger {
    OnClick,
    AfterPrevious,
    WithPrevious,
    OnLoad,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum Easing {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
    Bounce,
    Elastic,
    Back,
    Cubic(f32, f32, f32, f32),  // cubic-bezier
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum AnimRepeat {
    #[default]
    Once,
    Count(u32),
    Forever,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Direction {
    Left,
    Right,
    Up,
    Down,
    UpperLeft,
    UpperRight,
    LowerLeft,
    LowerRight,
}

/// Slide transition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transition {
    pub kind: TransitionKind,
    pub duration_ms: u32,
    pub direction: Option<Direction>,
    pub auto_advance_ms: Option<u32>,
    pub sound: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransitionKind {
    None,
    Fade,
    Push,
    Wipe,
    Split,
    Wheel,
    Random,
    Zoom,
    Flip,
    Cube,
    Gallery,
    Orbit,
}

/// Presentation theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
    pub effects: ThemeEffects,
}

impl Default for Theme {
    fn default() -> Self {
        Theme {
            name: "Office Theme".into(),
            colors: ThemeColors::default(),
            fonts: ThemeFonts::default(),
            effects: ThemeEffects::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    pub dark1: Color,
    pub light1: Color,
    pub dark2: Color,
    pub light2: Color,
    pub accent1: Color,
    pub accent2: Color,
    pub accent3: Color,
    pub accent4: Color,
    pub accent5: Color,
    pub accent6: Color,
    pub hyperlink: Color,
    pub followed_hyperlink: Color,
}

impl Default for ThemeColors {
    fn default() -> Self {
        ThemeColors {
            dark1: Color::from_hex("#000000").unwrap(),
            light1: Color::from_hex("#FFFFFF").unwrap(),
            dark2: Color::from_hex("#44546A").unwrap(),
            light2: Color::from_hex("#E7E6E6").unwrap(),
            accent1: Color::from_hex("#4472C4").unwrap(),
            accent2: Color::from_hex("#ED7D31").unwrap(),
            accent3: Color::from_hex("#A9D18E").unwrap(),
            accent4: Color::from_hex("#FFC000").unwrap(),
            accent5: Color::from_hex("#5B9BD5").unwrap(),
            accent6: Color::from_hex("#70AD47").unwrap(),
            hyperlink: Color::from_hex("#0563C1").unwrap(),
            followed_hyperlink: Color::from_hex("#954F72").unwrap(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeFonts {
    pub heading_family: String,
    pub body_family: String,
}

impl Default for ThemeFonts {
    fn default() -> Self {
        ThemeFonts {
            heading_family: "Calibri Light".into(),
            body_family: "Calibri".into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThemeEffects {
    pub shadow: bool,
    pub reflection: bool,
    pub glow: bool,
    pub soft_edges: bool,
}

/// Notes master
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct NotesMaster {
    pub background: Option<Background>,
    pub font: Option<FontSpec>,
}
