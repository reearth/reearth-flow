pub mod aggregator;
pub mod policy;
pub mod types;

pub use aggregator::*;
pub use policy::*;
pub use types::*;

include!(concat!(env!("OUT_DIR"), "/error_codes.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn error_code_roundtrips_through_serde_as_dotted_string() {
        let json = serde_json::to_string(&ErrorCode::Cesium3dtilesEmptyGeometry).unwrap();
        assert_eq!(json, "\"cesium3dtiles.empty_geometry\"");
        let back: ErrorCode = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ErrorCode::Cesium3dtilesEmptyGeometry);
    }

    #[test]
    fn every_code_has_registry_metadata() {
        // SSOT guard: category/default_disposition/message come from the registry for every code.
        for code in ErrorCode::ALL {
            assert!(!code.as_str().is_empty());
            assert!(code.as_str().contains('.'));
            assert!(!code.default_message().is_empty());
            // exercising category()/default_disposition() proves the generated tables are total
            let _ = code.category();
            let _ = code.default_disposition();
        }
        assert_eq!(
            ErrorCode::ALL.len(),
            15,
            "update this count when adding registry codes"
        );
    }

    #[test]
    fn known_code_matches_registry_entry() {
        let code = ErrorCode::GltfZeroFaceSolid;
        assert_eq!(code.as_str(), "gltf.zero_face_solid");
        assert_eq!(code.category(), ErrorCategory::Geometry);
        assert_eq!(code.default_disposition(), Disposition::WarnDrop);
    }

    #[test]
    fn from_draft_stamps_registry_fields_and_defaults() {
        let d = Diagnostic::from_draft(
            DiagnosticDraft::new(ErrorCode::Cesium3dtilesEmptyGeometry),
            Some("node-1".into()),
            Some("Cesium 3D Tiles Writer".into()),
            None,
        );
        assert_eq!(d.category, ErrorCategory::Geometry);
        assert_eq!(d.default_disposition, Disposition::WarnDrop);
        assert_eq!(d.effective_disposition, None);
        assert_eq!(d.severity, Severity::Warn);
        assert_eq!(d.message, "skipped feature with empty geometry");
        assert!(d.help.is_some());
    }

    #[test]
    fn from_draft_fatal_code_defaults_to_fatal_severity() {
        let d = Diagnostic::from_draft(
            DiagnosticDraft::new(ErrorCode::InternalInvariantViolation),
            None,
            None,
            None,
        );
        assert_eq!(d.severity, Severity::Fatal);
    }

    #[test]
    fn draft_overrides_beat_registry_defaults() {
        let d = Diagnostic::from_draft(
            DiagnosticDraft::new(ErrorCode::GltfZeroFaceSolid)
                .with_message("custom")
                .with_severity(Severity::Info),
            None,
            None,
            None,
        );
        assert_eq!(d.message, "custom");
        assert_eq!(d.severity, Severity::Info);
    }

    #[test]
    fn display_renders_the_console_line() {
        let d = Diagnostic::from_draft(
            DiagnosticDraft::new(ErrorCode::Cesium3dtilesEmptyGeometry),
            Some("3dtiles-writer-uuid".into()),
            Some("Cesium 3D Tiles Writer".into()),
            None,
        );
        assert_eq!(
            d.to_string(),
            "[WARN] cesium3dtiles.empty_geometry @ 3dtiles-writer-uuid (Cesium 3D Tiles Writer): skipped feature with empty geometry"
        );
        // it is a std error, so `?` coerces it into the actions' BoxedError signatures
        let boxed: Box<dyn std::error::Error + Send + Sync> = Box::new(d);
        assert!(boxed.to_string().starts_with("[WARN]"));
    }
}
