#[cfg(test)]
mod tests {
    #[test]
    fn test_czml_reader_registered() {
        // Just verify the CzmlReader is registered in the action mappings
        use reearth_flow_action_source::mapping::ACTION_FACTORY_MAPPINGS;

        let has_czml_reader = ACTION_FACTORY_MAPPINGS
            .keys()
            .any(|key| key == "CzmlReader");

        assert!(
            has_czml_reader,
            "CzmlReader should be registered in ACTION_FACTORY_MAPPINGS"
        );
    }
}
