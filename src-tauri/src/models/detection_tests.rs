#[cfg(test)]
mod tests {
    use crate::models::{DetectionResult, DetectionStep};
    use serde_json;

    /// Test DetectionResult serialization to snake_case
    #[test]
    fn test_detection_result_serialization() {
        // Test Found
        let result = DetectionResult::Found;
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(json, "\"found\"");

        // Test NotFound
        let result = DetectionResult::NotFound;
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(json, "\"not_found\"");

        // Test Error
        let result = DetectionResult::Error;
        let json = serde_json::to_string(&result).unwrap();
        assert_eq!(json, "\"error\"");
    }

    /// Test DetectionResult deserialization from snake_case
    #[test]
    fn test_detection_result_deserialization() {
        // Test Found
        let result: DetectionResult = serde_json::from_str("\"found\"").unwrap();
        assert_eq!(result, DetectionResult::Found);

        // Test NotFound
        let result: DetectionResult = serde_json::from_str("\"not_found\"").unwrap();
        assert_eq!(result, DetectionResult::NotFound);

        // Test Error
        let result: DetectionResult = serde_json::from_str("\"error\"").unwrap();
        assert_eq!(result, DetectionResult::Error);
    }

    /// Test DetectionResult PartialEq
    #[test]
    fn test_detection_result_equality() {
        assert_eq!(DetectionResult::Found, DetectionResult::Found);
        assert_eq!(DetectionResult::NotFound, DetectionResult::NotFound);
        assert_eq!(DetectionResult::Error, DetectionResult::Error);
        assert_ne!(DetectionResult::Found, DetectionResult::NotFound);
        assert_ne!(DetectionResult::Found, DetectionResult::Error);
        assert_ne!(DetectionResult::NotFound, DetectionResult::Error);
    }

    /// Test DetectionStep serialization and deserialization
    #[test]
    fn test_detection_step_serialization() {
        let step = DetectionStep {
            phase: "Phase 1: npm global prefix".to_string(),
            action: "Checking npm prefix".to_string(),
            target: "/usr/local/bin/openclaw".to_string(),
            result: DetectionResult::Found,
            message: None,
        };

        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"phase\":\"Phase 1: npm global prefix\""));
        assert!(json.contains("\"action\":\"Checking npm prefix\""));
        assert!(json.contains("\"target\":\"/usr/local/bin/openclaw\""));
        assert!(json.contains("\"result\":\"found\""));
        assert!(json.contains("\"message\":null"));

        // Deserialize back
        let decoded: DetectionStep = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.phase, step.phase);
        assert_eq!(decoded.action, step.action);
        assert_eq!(decoded.target, step.target);
        assert_eq!(decoded.result, step.result);
        assert_eq!(decoded.message, step.message);
    }

    /// Test DetectionStep with message (Error case)
    #[test]
    fn test_detection_step_with_message() {
        let step = DetectionStep {
            phase: "Phase 1: npm global prefix".to_string(),
            action: "Checking npm prefix".to_string(),
            target: "npm config get prefix".to_string(),
            result: DetectionResult::Error,
            message: Some("Failed to get npm global prefix".to_string()),
        };

        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"result\":\"error\""));
        assert!(json.contains("\"message\":\"Failed to get npm global prefix\""));

        // Deserialize back
        let decoded: DetectionStep = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.result, DetectionResult::Error);
        assert_eq!(decoded.message, Some("Failed to get npm global prefix".to_string()));
    }

    /// Test DetectionStep with NotFound result
    #[test]
    fn test_detection_step_not_found() {
        let step = DetectionStep {
            phase: "Phase 2: Hardcoded paths".to_string(),
            action: "Checking path".to_string(),
            target: "C:\\Program Files\\nodejs\\openclaw.cmd".to_string(),
            result: DetectionResult::NotFound,
            message: None,
        };

        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("\"result\":\"not_found\""));

        // Deserialize back
        let decoded: DetectionStep = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.result, DetectionResult::NotFound);
        assert_eq!(decoded.message, None);
    }

    /// Test array of DetectionStep serialization
    #[test]
    fn test_detection_step_array() {
        let steps = vec![
            DetectionStep {
                phase: "Phase 1: npm global prefix".to_string(),
                action: "Checking npm prefix".to_string(),
                target: "/path/1".to_string(),
                result: DetectionResult::NotFound,
                message: None,
            },
            DetectionStep {
                phase: "Phase 2: Hardcoded paths".to_string(),
                action: "Checking path".to_string(),
                target: "/path/2".to_string(),
                result: DetectionResult::Found,
                message: None,
            },
        ];

        let json = serde_json::to_string(&steps).unwrap();
        assert!(json.starts_with('['));
        assert!(json.ends_with(']'));

        // Deserialize back
        let decoded: Vec<DetectionStep> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.len(), 2);
        assert_eq!(decoded[0].result, DetectionResult::NotFound);
        assert_eq!(decoded[1].result, DetectionResult::Found);
    }

    /// Test DetectionStep Debug trait
    #[test]
    fn test_detection_step_debug() {
        let step = DetectionStep {
            phase: "Phase 1".to_string(),
            action: "Check".to_string(),
            target: "target".to_string(),
            result: DetectionResult::Found,
            message: None,
        };

        let debug_str = format!("{:?}", step);
        assert!(debug_str.contains("DetectionStep"));
        assert!(debug_str.contains("Phase 1"));
        assert!(debug_str.contains("Found"));
    }

    /// Test DetectionResult Debug trait
    #[test]
    fn test_detection_result_debug() {
        assert!(format!("{:?}", DetectionResult::Found).contains("Found"));
        assert!(format!("{:?}", DetectionResult::NotFound).contains("NotFound"));
        assert!(format!("{:?}", DetectionResult::Error).contains("Error"));
    }

    /// Test DetectionStep Clone trait
    #[test]
    fn test_detection_step_clone() {
        let step = DetectionStep {
            phase: "Phase 1".to_string(),
            action: "Check".to_string(),
            target: "target".to_string(),
            result: DetectionResult::Found,
            message: Some("msg".to_string()),
        };

        let cloned = step.clone();
        assert_eq!(step.phase, cloned.phase);
        assert_eq!(step.action, cloned.action);
        assert_eq!(step.target, cloned.target);
        assert_eq!(step.result, cloned.result);
        assert_eq!(step.message, cloned.message);
    }

    /// Test DetectionResult Clone trait
    #[test]
    fn test_detection_result_clone() {
        let result = DetectionResult::Found;
        let cloned = result.clone();
        assert_eq!(result, cloned);
    }
}
