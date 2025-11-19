use aphrodite_core::rendering::{ChartSpec, Color, Point, Shape};

#[test]
fn test_chartspec_new() {
    let spec = ChartSpec::new(800.0, 600.0);
    
    assert_eq!(spec.width, 800.0);
    assert_eq!(spec.height, 600.0);
    assert_eq!(spec.center.x, 400.0);
    assert_eq!(spec.center.y, 300.0);
}

#[test]
fn test_color_from_hex() {
    let color = Color::from_hex("#FF0000");
    assert!(color.is_some());
    let color = color.unwrap();
    assert_eq!(color.r, 255);
    assert_eq!(color.g, 0);
    assert_eq!(color.b, 0);
    assert_eq!(color.a, 255);
}

#[test]
fn test_color_to_css_string() {
    let color = Color {
        r: 255,
        g: 0,
        b: 0,
        a: 255,
    };
    
    let css = color.to_css_string();
    assert!(css.contains("rgb(255, 0, 0)"));
}

#[test]
fn test_shape_circle_serialization() {
    let shape = Shape::Circle {
        center: Point { x: 100.0, y: 200.0 },
        radius: 50.0,
        fill: Some(Color::WHITE),
        stroke: None,
    };
    
    // Test that it can be serialized
    let json = serde_json::to_string(&shape);
    assert!(json.is_ok());
}

