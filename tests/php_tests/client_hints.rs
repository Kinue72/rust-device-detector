use anyhow::Result;

use rust_device_detector::client_hints::ClientHint;
use rust_device_detector::device_detector::DeviceDetector;

#[test]
fn test_form_factors_parsing() -> Result<()> {
    // Test FormFactors header parsing like in PHP tests
    let headers = vec![
        ("sec-ch-ua-form-factors".to_string(), r#""Desktop""#.to_string()),
    ];

    let client_hint = ClientHint::from_headers(headers)?;
    assert_eq!(client_hint.form_factors, vec!["desktop"]);

    // Test multiple form factors
    let headers = vec![
        ("sec-ch-ua-form-factors".to_string(), r#""Mobile", "Touch""#.to_string()),
    ];

    let client_hint = ClientHint::from_headers(headers)?;
    assert_eq!(client_hint.form_factors, vec!["mobile", "touch"]);

    Ok(())
}

#[test]
fn test_form_factors_device_detection() -> Result<()> {
    // Test that FormFactors correctly detects device type
    let detector = DeviceDetector::new();
    
    let headers = vec![
        ("sec-ch-ua-form-factors".to_string(), r#""Desktop""#.to_string()),
    ];
    
    let result = detector.parse("", Some(headers))?;
    
    let device_type: Option<&str> = result
        .get_known_device()
        .and_then(|dev| dev.device.as_ref())
        .and_then(|dev| dev.device_type.as_ref())
        .map(|t| t.as_str());
    
    // Should detect desktop device type from FormFactors
    assert_eq!(device_type, Some("desktop"));

    Ok(())
}