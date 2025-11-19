use crate::canvas::render_shape;
use aphrodite_core::rendering::{ChartSpec, Shape};
use serde_json;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

/// Chart renderer for WASM
#[wasm_bindgen]
pub struct ChartRenderer {
    spec: ChartSpec,
}

#[wasm_bindgen]
impl ChartRenderer {
    /// Create a new renderer from JSON ChartSpec
    #[wasm_bindgen(constructor)]
    pub fn new(spec_json: &str) -> Result<ChartRenderer, JsValue> {
        let spec: ChartSpec = serde_json::from_str(spec_json)
            .map_err(|e| JsValue::from_str(&format!("Failed to parse ChartSpec: {}", e)))?;
        Ok(ChartRenderer { spec })
    }

    /// Render the chart to an HTML5 Canvas
    #[wasm_bindgen]
    pub fn render_to_canvas(&self, ctx: &CanvasRenderingContext2d) -> Result<(), JsValue> {
        // Clear canvas
        ctx.clear_rect(0.0, 0.0, self.spec.width as f64, self.spec.height as f64);

        // Set background color
        let bg_color = &self.spec.background_color;
        ctx.set_fill_style(&format!("rgba({}, {}, {}, {})", 
            bg_color.r, bg_color.g, bg_color.b, bg_color.a as f32 / 255.0));
        ctx.fill_rect(0.0, 0.0, self.spec.width as f64, self.spec.height as f64);

        // Render each shape
        for shape in &self.spec.shapes {
            render_shape(ctx, shape)?;
        }

        Ok(())
    }

    /// Convert ChartSpec to SVG string
    #[wasm_bindgen]
    pub fn to_svg(&self) -> String {
        let mut svg = format!(
            r#"<svg width="{}" height="{}" xmlns="http://www.w3.org/2000/svg">"#,
            self.spec.width, self.spec.height
        );

        // Set background
        let bg = &self.spec.background_color;
        svg.push_str(&format!(
            r#"<rect width="100%" height="100%" fill="rgba({}, {}, {}, {})"/>"#,
            bg.r, bg.g, bg.b, bg.a as f32 / 255.0
        ));

        // Render shapes as SVG elements
        for shape in &self.spec.shapes {
            svg.push_str(&shape_to_svg(shape));
        }

        svg.push_str("</svg>");
        svg
    }
}

/// Convert a shape to SVG string
fn shape_to_svg(shape: &Shape) -> String {
    match shape {
        Shape::Circle { center, radius, fill, stroke, .. } => {
            let fill_attr = fill.map(|c| format!("fill=\"rgba({}, {}, {}, {})\"", 
                c.r, c.g, c.b, c.a as f32 / 255.0)).unwrap_or_else(|| "fill=\"none\"".to_string());
            let stroke_attr = stroke.as_ref().map(|s| format!("stroke=\"rgba({}, {}, {}, {})\" stroke-width=\"{}\"", 
                s.color.r, s.color.g, s.color.b, s.color.a as f32 / 255.0, s.width)).unwrap_or_else(|| String::new());
            format!(r#"<circle cx="{}" cy="{}" r="{}" {} {} />"#, 
                center.x, center.y, radius, fill_attr, stroke_attr)
        }
        Shape::Arc { center, radius_inner, radius_outer, start_angle, end_angle, fill, stroke, .. } => {
            // Convert arc to SVG path
            let start_rad = start_angle.to_radians();
            let end_rad = end_angle.to_radians();
            let x1 = center.x + radius_outer * start_rad.cos();
            let y1 = center.y + radius_outer * start_rad.sin();
            let x2 = center.x + radius_outer * end_rad.cos();
            let y2 = center.y + radius_outer * end_rad.sin();
            let x3 = center.x + radius_inner * end_rad.cos();
            let y3 = center.y + radius_inner * end_rad.sin();
            let x4 = center.x + radius_inner * start_rad.cos();
            let y4 = center.y + radius_inner * start_rad.sin();
            
            let large_arc = if (end_angle - start_angle) > 180.0 { 1 } else { 0 };
            let fill_attr = fill.map(|c| format!("fill=\"rgba({}, {}, {}, {})\"", 
                c.r, c.g, c.b, c.a as f32 / 255.0)).unwrap_or_else(|| "fill=\"none\"".to_string());
            let stroke_attr = stroke.as_ref().map(|s| format!("stroke=\"rgba({}, {}, {}, {})\" stroke-width=\"{}\"", 
                s.color.r, s.color.g, s.color.b, s.color.a as f32 / 255.0, s.width)).unwrap_or_else(|| String::new());
            
            format!(r#"<path d="M {} {} A {} {} 0 {} 1 {} {} L {} {} A {} {} 0 {} 0 {} {} Z" {} {} />"#,
                x1, y1, radius_outer, radius_outer, large_arc, x2, y2,
                x3, y3, radius_inner, radius_inner, large_arc, x4, y4,
                fill_attr, stroke_attr)
        }
        Shape::Line { from, to, stroke } => {
            format!(r#"<line x1="{}" y1="{}" x2="{}" y2="{}" stroke="rgba({}, {}, {}, {})" stroke-width="{}" />"#,
                from.x, from.y, to.x, to.y,
                stroke.color.r, stroke.color.g, stroke.color.b, stroke.color.a as f32 / 255.0,
                stroke.width)
        }
        Shape::Text { position, content, size, color, .. } => {
            format!(r#"<text x="{}" y="{}" font-size="{}" fill="rgba({}, {}, {}, {})">{}</text>"#,
                position.x, position.y, size,
                color.r, color.g, color.b, color.a as f32 / 255.0,
                content)
        }
        _ => String::new(), // Placeholder for other shapes
    }
}

