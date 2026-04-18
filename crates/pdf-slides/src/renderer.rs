//! SVG renderer for slides — converts SlideElement tree to SVG markup.

use pdf_core::presentation::{
    Slide, SlideElement, Background, ShapeKind, ShapeFill, TextBoxEl, ImageEl,
    ShapeEl, SlideParagraph, SlideRun
};
use pdf_core::common::Color;

pub struct SvgRenderer {
    width: f32,
    height: f32,
}

impl SvgRenderer {
    pub fn new(width: f32, height: f32) -> Self {
        SvgRenderer { width, height }
    }

    pub fn render(&self, slide: &Slide) -> String {
        let mut svg = String::new();
        svg.push_str(&format!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="{}" height="{}" viewBox="0 0 {} {}">"#,
            self.width, self.height, self.width, self.height
        ));

        // Background
        svg.push_str(&self.render_background(&slide.background));

        // Render elements in order
        for element in &slide.elements {
            svg.push_str(&self.render_element(element));
        }

        svg.push_str("</svg>");
        svg
    }

    fn render_background(&self, bg: &Background) -> String {
        match bg {
            Background::Solid(color) => {
                format!(r#"<rect width="{}" height="{}" fill="{}"/>"#,
                    self.width, self.height, color_to_css(color))
            }
            Background::Gradient(g) => {
                let stops: String = g.stops.iter().enumerate().map(|(i, stop)| {
                    format!(r#"<stop offset="{:.1}%" stop-color="{}"/>"#,
                        stop.position * 100.0, color_to_css(&stop.color))
                }).collect();
                let grad_id = "bg_grad";
                let (x2, y2) = (
                    (g.angle.to_radians().cos() * 100.0 + 100.0) / 2.0,
                    (g.angle.to_radians().sin() * 100.0 + 100.0) / 2.0,
                );
                format!(
                    r#"<defs><linearGradient id="{}" x1="0%" y1="0%" x2="{}%" y2="{}%">{}</linearGradient></defs><rect width="{}" height="{}" fill="url(#{})" />"#,
                    grad_id, x2, y2, stops, self.width, self.height, grad_id
                )
            }
            Background::Image { data, mime_type, .. } => {
                let b64 = base64_encode(data);
                format!(
                    r#"<image href="data:{};base64,{}" width="{}" height="{}" preserveAspectRatio="xMidYMid slice"/>"#,
                    mime_type, b64, self.width, self.height
                )
            }
            Background::Theme(_) => {
                format!("<rect width=\"{}\" height=\"{}\" fill=\"#FFFFFF\"/>", self.width, self.height)
            }
        }
    }

    fn render_element(&self, el: &SlideElement) -> String {
        match el {
            SlideElement::TextBox(tb) => self.render_textbox(tb),
            SlideElement::Image(img) => self.render_image(img),
            SlideElement::Shape(shape) => self.render_shape(shape),
            SlideElement::Group(group) => {
                let inner: String = group.children.iter().map(|c| self.render_element(c)).collect();
                format!(
                    r#"<g transform="translate({} {}) rotate({} {} {})">{}</g>"#,
                    group.transform.x, group.transform.y,
                    group.transform.rotation,
                    group.transform.width / 2.0, group.transform.height / 2.0,
                    inner
                )
            }
            _ => String::new(),
        }
    }

    fn render_textbox(&self, tb: &TextBoxEl) -> String {
        let t = &tb.transform;
        let mut content = format!(
            r#"<g transform="translate({} {}) rotate({} {} {})">"#,
            t.x, t.y, t.rotation, t.width / 2.0, t.height / 2.0
        );

        // Background fill
        if let Some(fill) = &tb.fill {
            content.push_str(&format!(
                r#"<rect width="{}" height="{}" fill="{}"/>"#,
                t.width, t.height, color_to_css(fill)
            ));
        }

        // Render paragraphs
        let mut y = tb.padding.top + 12.0;
        for para in &tb.paragraphs {
            let text_content = para.runs.iter().map(|r| escape_xml(&r.text)).collect::<String>();
            if !text_content.is_empty() {
                let font_size = para.runs.first()
                    .and_then(|r| r.font.as_ref())
                    .map(|f| f.size)
                    .unwrap_or(14.0);
                let color = para.runs.first()
                    .and_then(|r| r.color.as_ref())
                    .map(color_to_css)
                    .unwrap_or_else(|| "#000000".to_string());
                let bold = para.runs.first().map(|r| r.bold).unwrap_or(false);
                let italic = para.runs.first().map(|r| r.italic).unwrap_or(false);

                let font_weight = if bold { "bold" } else { "normal" };
                let font_style = if italic { "italic" } else { "normal" };

                content.push_str(&format!(
                    r#"<text x="{}" y="{}" font-size="{}" fill="{}" font-weight="{}" font-style="{}" text-anchor="{}">{}</text>"#,
                    tb.padding.left,
                    y,
                    font_size,
                    color,
                    font_weight,
                    font_style,
                    match para.align {
                        pdf_core::TextAlign::Left => "start",
                        pdf_core::TextAlign::Center => "middle",
                        pdf_core::TextAlign::Right => "end",
                        pdf_core::TextAlign::Justify => "start",
                    },
                    text_content
                ));
                y += font_size * para.line_height;
            }
        }

        content.push_str("</g>");
        content
    }

    fn render_image(&self, img: &ImageEl) -> String {
        let t = &img.transform;
        let b64 = base64_encode(&img.data);
        format!(
            r#"<image x="{}" y="{}" width="{}" height="{}" href="data:{};base64,{}" transform="rotate({} {} {})" preserveAspectRatio="xMidYMid meet"/>"#,
            t.x, t.y, t.width, t.height,
            img.mime_type, b64,
            t.rotation, t.x + t.width / 2.0, t.y + t.height / 2.0
        )
    }

    fn render_shape(&self, shape: &ShapeEl) -> String {
        let t = &shape.transform;
        let fill_css = match &shape.fill {
            ShapeFill::None => "none".to_string(),
            ShapeFill::Solid(c) => color_to_css(c),
            ShapeFill::Gradient(_) => "#888888".to_string(), // simplified
            ShapeFill::Pattern => "url(#pattern)".to_string(),
        };
        let stroke = shape.stroke.as_ref()
            .map(|s| format!(r#"stroke="{}" stroke-width="{}""#, color_to_css(&s.color), s.width))
            .unwrap_or_default();

        let shape_svg = match &shape.kind {
            ShapeKind::Rectangle => format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" {} />"#,
                t.x, t.y, t.width, t.height, fill_css, stroke
            ),
            ShapeKind::RoundedRectangle { radius } => format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" rx="{}" fill="{}" {} />"#,
                t.x, t.y, t.width, t.height, radius, fill_css, stroke
            ),
            ShapeKind::Ellipse => format!(
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" fill="{}" {} />"#,
                t.x + t.width / 2.0, t.y + t.height / 2.0, t.width / 2.0, t.height / 2.0, fill_css, stroke
            ),
            ShapeKind::Line => format!(
                r#"<line x1="{}" y1="{}" x2="{}" y2="{}" {} />"#,
                t.x, t.y, t.x + t.width, t.y + t.height, stroke
            ),
            ShapeKind::Triangle => {
                let pts = format!("{},{} {},{} {},{}",
                    t.x + t.width / 2.0, t.y,
                    t.x, t.y + t.height,
                    t.x + t.width, t.y + t.height);
                format!(r#"<polygon points="{}" fill="{}" {} />"#, pts, fill_css, stroke)
            }
            ShapeKind::Custom(path) => format!(
                r#"<path d="{}" fill="{}" {} transform="translate({} {})" />"#,
                path, fill_css, stroke, t.x, t.y
            ),
            _ => format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" fill="{}" {} />"#,
                t.x, t.y, t.width, t.height, fill_css, stroke
            ),
        };

        if t.rotation != 0.0 {
            format!(
                r#"<g transform="rotate({} {} {})">{}</g>"#,
                t.rotation, t.x + t.width / 2.0, t.y + t.height / 2.0, shape_svg
            )
        } else {
            shape_svg
        }
    }
}

fn color_to_css(c: &Color) -> String {
    if c.a == 255 {
        format!("#{:02X}{:02X}{:02X}", c.r, c.g, c.b)
    } else {
        format!("rgba({},{},{},{:.3})", c.r, c.g, c.b, c.a as f32 / 255.0)
    }
}

fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
     .replace('<', "&lt;")
     .replace('>', "&gt;")
     .replace('"', "&quot;")
     .replace('\'', "&#39;")
}

fn base64_encode(data: &[u8]) -> String {
    // Simple base64 implementation to avoid adding a dep here
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity((data.len() + 2) / 3 * 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        result.push(ALPHABET[(n >> 18 & 63) as usize] as char);
        result.push(ALPHABET[(n >> 12 & 63) as usize] as char);
        result.push(if chunk.len() > 1 { ALPHABET[(n >> 6 & 63) as usize] as char } else { '=' });
        result.push(if chunk.len() > 2 { ALPHABET[(n & 63) as usize] as char } else { '=' });
    }
    result
}
