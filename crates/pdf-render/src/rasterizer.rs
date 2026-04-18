//! Rasterizer — converts LayoutPage trees to CanvasCmd streams and PNG images.

use pdf_writer::layout::{LayoutPage, LayoutBox, LayoutBoxKind};
use crate::canvas::{PageRender, CanvasCmd, FillRule};
use pdf_core::common::{Color, FontSpec, FontWeight, FontStyle};
use tiny_skia::*;

/// Render a LayoutPage to a stream of CanvasCmd for the browser frontend
pub fn render_to_canvas_commands(page: &LayoutPage) -> PageRender {
    let mut render = PageRender::new(page.page_num, page.width, page.height);

    // Background
    let bg = page.background.clone().unwrap_or(Color::WHITE);
    render.fill_rect(0.0, 0.0, page.width, page.height, bg);

    for layout_box in &page.boxes {
        render_layout_box(&mut render, layout_box);
    }

    render
}

fn render_layout_box(render: &mut PageRender, lb: &LayoutBox) {
    match &lb.kind {
        LayoutBoxKind::GlyphRun { text, font, color } => {
            render.fill_text(text.clone(), lb.x, lb.y + lb.height * 0.7, font.clone(), color.clone());
        }
        LayoutBoxKind::HorizontalRule => {
            render.set_line_width(0.5);
            render.begin_path();
            render.move_to(lb.x, lb.y);
            render.line_to(lb.x + lb.width, lb.y);
            render.push(CanvasCmd::Stroke);
        }
        LayoutBoxKind::Block | LayoutBoxKind::Page { .. } => {
            for child in &lb.children {
                render_layout_box(render, child);
            }
        }
        LayoutBoxKind::TableCell { .. } => {
            // Draw cell border
            render.push(CanvasCmd::StrokeRect {
                x: lb.x, y: lb.y, w: lb.width, h: lb.height,
                color: Color { r: 200, g: 200, b: 200, a: 255 },
                line_width: 0.5,
            });
            for child in &lb.children {
                render_layout_box(render, child);
            }
        }
        _ => {
            for child in &lb.children {
                render_layout_box(render, child);
            }
        }
    }
}

/// Rasterize a LayoutPage to an RGBA image using tiny-skia
pub fn rasterize_page(page: &LayoutPage, scale: f32) -> Option<tiny_skia::Pixmap> {
    let w = (page.width * scale) as u32;
    let h = (page.height * scale) as u32;
    let mut pixmap = Pixmap::new(w, h)?;

    // White background
    pixmap.fill(tiny_skia::Color::from_rgba8(255, 255, 255, 255));

    let mut paint = Paint::default();
    paint.set_color_rgba8(0, 0, 0, 255);
    paint.anti_alias = true;

    let commands = render_to_canvas_commands(page);

    // Replay canvas commands onto the pixmap
    let mut transform = Transform::identity();

    for cmd in &commands.commands {
        match cmd {
            CanvasCmd::FillRect { x, y, w, h, color } => {
                let mut p = Paint::default();
                p.set_color_rgba8(color.r, color.g, color.b, color.a);

                let rect = Rect::from_xywh(x * scale, y * scale, w * scale, h * scale)?;
                pixmap.fill_rect(rect, &p, Transform::identity(), None);
            }
            CanvasCmd::StrokeRect { x, y, w, h, color, line_width } => {
                let mut stroke = Stroke::default();
                stroke.width = line_width * scale;

                let mut p = Paint::default();
                p.set_color_rgba8(color.r, color.g, color.b, color.a);

                let path = PathBuilder::from_rect(
                    Rect::from_xywh(x * scale, y * scale, w * scale, h * scale)?
                );
                pixmap.stroke_path(&path, &p, &stroke, Transform::identity(), None);
            }
            CanvasCmd::FillText { text, x, y, font, color, .. } => {
                // Text rendering in tiny-skia requires a font loader
                // For Phase 1 we draw a placeholder line where text would be
                let mut p = Paint::default();
                p.set_color_rgba8(color.r, color.g, color.b, color.a);
                let font_h = font.size * scale / 72.0 * 96.0;  // pt to px
                let text_len = text.len() as f32 * font_h * 0.5;
                if let Some(rect) = Rect::from_xywh(x * scale, (y - font_h) * scale, text_len, font_h * 0.1) {
                    pixmap.fill_rect(rect, &p, Transform::identity(), None);
                }
            }
            _ => {}
        }
    }

    Some(pixmap)
}

/// Rasterize and encode to PNG bytes
pub fn rasterize_to_png(page: &LayoutPage, scale: f32) -> Option<Vec<u8>> {
    let pixmap = rasterize_page(page, scale)?;
    pixmap.encode_png().ok()
}
