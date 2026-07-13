pub mod types;

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
        assert_eq!(ErrorCode::ALL.len(), 7);
    }

    #[test]
    fn known_code_matches_registry_entry() {
        let code = ErrorCode::GltfZeroFaceSolid;
        assert_eq!(code.as_str(), "gltf.zero_face_solid");
        assert_eq!(code.category(), ErrorCategory::Geometry);
        assert_eq!(code.default_disposition(), Disposition::WarnDrop);
    }
}
