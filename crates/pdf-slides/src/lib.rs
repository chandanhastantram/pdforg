//! pdf-slides — Presentation engine with SVG renderer and animation model.

pub mod renderer;
pub use renderer::*;

pub use pdf_core::presentation::*;

/// Entry point: convert a slide to SVG
pub fn slide_to_svg(slide: &pdf_core::Slide, width: f32, height: f32) -> String {
    renderer::SvgRenderer::new(width, height).render(slide)
}
