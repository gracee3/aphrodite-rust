use aphrodite_core::layout::load_wheel_definition_from_json;

#[test]
fn test_load_wheel_definition_valid() {
    let json = r#"
    {
      "name": "Test Wheel",
      "rings": [
        {
          "slug": "ring_signs",
          "type": "signs",
          "label": "Zodiac Signs",
          "orderIndex": 0,
          "radiusInner": 0.85,
          "radiusOuter": 1.0,
          "dataSource": {
            "kind": "static_zodiac"
          }
        }
      ]
    }
    "#;
    
    let result = load_wheel_definition_from_json(json);
    assert!(result.is_ok());
    let wheel = result.unwrap();
    assert_eq!(wheel.wheel.name, "Test Wheel");
    assert_eq!(wheel.wheel.rings.len(), 1);
}

#[test]
fn test_load_wheel_definition_invalid() {
    let json = r#"
    {
      "name": "Test Wheel"
    }
    "#;
    
    let result = load_wheel_definition_from_json(json);
    assert!(result.is_err());
}

#[test]
fn test_load_wheel_definition_missing_rings() {
    let json = r#"
    {
      "name": "Test Wheel",
      "rings": []
    }
    "#;
    
    let result = load_wheel_definition_from_json(json);
    assert!(result.is_err());
}

