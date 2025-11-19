use aphrodite_core::rendering::{Color, Shape, Stroke};
use web_sys::CanvasRenderingContext2d;

/// Render a shape to HTML5 Canvas
pub fn render_shape(ctx: &CanvasRenderingContext2d, shape: &Shape) -> Result<(), wasm_bindgen::JsValue> {
    match shape {
        Shape::Circle { center, radius, fill, stroke } => {
            ctx.begin_path();
            ctx.arc(
                center.x as f64,
                center.y as f64,
                *radius as f64,
                0.0,
                2.0 * std::f64::consts::PI,
            )?;
            
            if let Some(fill_color) = fill {
                ctx.set_fill_style(&color_to_css(fill_color));
                ctx.fill()?;
            }
            
            if let Some(stroke_style) = stroke {
                ctx.set_stroke_style(&color_to_css(&stroke_style.color));
                ctx.set_line_width(stroke_style.width as f64);
                ctx.stroke()?;
            }
        }
        Shape::Arc { center, radius_inner, radius_outer, start_angle, end_angle, fill, stroke } => {
            // Render arc as a path
            ctx.begin_path();
            let start_rad = start_angle.to_radians() as f64;
            let end_rad = end_angle.to_radians() as f64;
            
            // Outer arc
            ctx.arc(
                center.x as f64,
                center.y as f64,
                *radius_outer as f64,
                start_rad,
                end_rad,
            )?;
            
            // Line to inner arc
            let inner_end_x = center.x as f64 + *radius_inner as f64 * end_rad.cos();
            let inner_end_y = center.y as f64 + *radius_inner as f64 * end_rad.sin();
            ctx.line_to(inner_end_x, inner_end_y)?;
            
            // Inner arc (reverse direction)
            ctx.arc(
                center.x as f64,
                center.y as f64,
                *radius_inner as f64,
                end_rad,
                start_rad,
            )?;
            
            ctx.close_path();
            
            if let Some(fill_color) = fill {
                ctx.set_fill_style(&color_to_css(fill_color));
                ctx.fill()?;
            }
            
            if let Some(stroke_style) = stroke {
                ctx.set_stroke_style(&color_to_css(&stroke_style.color));
                ctx.set_line_width(stroke_style.width as f64);
                ctx.stroke()?;
            }
        }
        Shape::Line { from, to, stroke } => {
            ctx.begin_path();
            ctx.move_to(from.x as f64, from.y as f64);
            ctx.line_to(to.x as f64, to.y as f64);
            ctx.set_stroke_style(&color_to_css(&stroke.color));
            ctx.set_line_width(stroke.width as f64);
            ctx.stroke()?;
        }
        Shape::Text { position, content, size, color, .. } => {
            ctx.set_fill_style(&color_to_css(color));
            ctx.set_font(&format!("{}px sans-serif", size));
            ctx.fill_text(content, position.x as f64, position.y as f64)?;
        }
        Shape::PlanetGlyph { center, planet_id, size, color, .. } => {
            // Render planet glyph as text (using Unicode glyphs)
            ctx.set_fill_style(&color_to_css(color));
            ctx.set_font(&format!("{}px sans-serif", size));
            // For now, just render the planet ID - full implementation would use glyph fonts
            ctx.fill_text(planet_id, center.x as f64, center.y as f64)?;
        }
        Shape::AspectLine { from, to, aspect_type: _, color, width, .. } => {
            ctx.begin_path();
            ctx.move_to(from.x as f64, from.y as f64);
            ctx.line_to(to.x as f64, to.y as f64);
            ctx.set_stroke_style(&color_to_css(color));
            ctx.set_line_width(*width as f64);
            ctx.stroke()?;
        }
        Shape::HouseSegment { center, house_num: _, start_angle, end_angle, radius_inner, radius_outer, fill, stroke } => {
            // Similar to Arc rendering
            ctx.begin_path();
            let start_rad = start_angle.to_radians() as f64;
            let end_rad = end_angle.to_radians() as f64;
            
            ctx.arc(center.x as f64, center.y as f64, *radius_outer as f64, start_rad, end_rad)?;
            let inner_end_x = center.x as f64 + *radius_inner as f64 * end_rad.cos();
            let inner_end_y = center.y as f64 + *radius_inner as f64 * end_rad.sin();
            ctx.line_to(inner_end_x, inner_end_y)?;
            ctx.arc(center.x as f64, center.y as f64, *radius_inner as f64, end_rad, start_rad)?;
            ctx.close_path();
            
            ctx.set_fill_style(&color_to_css(fill));
            ctx.fill()?;
            
            if let Some(stroke_style) = stroke {
                ctx.set_stroke_style(&color_to_css(&stroke_style.color));
                ctx.set_line_width(stroke_style.width as f64);
                ctx.stroke()?;
            }
        }
        Shape::SignSegment { center, sign_index: _, start_angle, end_angle, radius_inner, radius_outer, fill, stroke } => {
            // Same as HouseSegment
            ctx.begin_path();
            let start_rad = start_angle.to_radians() as f64;
            let end_rad = end_angle.to_radians() as f64;
            
            ctx.arc(center.x as f64, center.y as f64, *radius_outer as f64, start_rad, end_rad)?;
            let inner_end_x = center.x as f64 + *radius_inner as f64 * end_rad.cos();
            let inner_end_y = center.y as f64 + *radius_inner as f64 * end_rad.sin();
            ctx.line_to(inner_end_x, inner_end_y)?;
            ctx.arc(center.x as f64, center.y as f64, *radius_inner as f64, end_rad, start_rad)?;
            ctx.close_path();
            
            ctx.set_fill_style(&color_to_css(fill));
            ctx.fill()?;
            
            if let Some(stroke_style) = stroke {
                ctx.set_stroke_style(&color_to_css(&stroke_style.color));
                ctx.set_line_width(stroke_style.width as f64);
                ctx.stroke()?;
            }
        }
        Shape::Path { points, closed, fill, stroke } => {
            if points.is_empty() {
                return Ok(());
            }
            
            ctx.begin_path();
            ctx.move_to(points[0].x as f64, points[0].y as f64);
            for point in points.iter().skip(1) {
                ctx.line_to(point.x as f64, point.y as f64);
            }
            if *closed {
                ctx.close_path();
            }
            
            if let Some(fill_color) = fill {
                ctx.set_fill_style(&color_to_css(fill_color));
                ctx.fill()?;
            }
            
            if let Some(stroke_style) = stroke {
                ctx.set_stroke_style(&color_to_css(&stroke_style.color));
                ctx.set_line_width(stroke_style.width as f64);
                ctx.stroke()?;
            }
        }
    }
    
    Ok(())
}

/// Convert Color to CSS string
fn color_to_css(color: &Color) -> String {
    if color.a == 255 {
        format!("rgb({}, {}, {})", color.r, color.g, color.b)
    } else {
        format!(
            "rgba({}, {}, {}, {})",
            color.r,
            color.g,
            color.b,
            color.a as f32 / 255.0
        )
    }
}

