// Temporary file to generate schema for HTTPCaller
// Run this with: cargo test --package reearth-flow-action-processor --lib http::schema_gen::print_schema -- --nocapture

#[cfg(test)]
mod tests {
    use crate::http::factory::HttpCallerFactory;
    use reearth_flow_runtime::node::ProcessorFactory;

    #[test]
    fn print_schema() {
        let factory = HttpCallerFactory;
        if let Some(schema) = factory.parameter_schema() {
            let json = serde_json::to_string_pretty(&schema).unwrap();
            println!("\n{json}\n");
            println!("Name: {}", factory.name());
            println!("Description: {}", factory.description());
            println!("Categories: {:?}", factory.categories());
            println!("Input Ports: {:?}", factory.get_input_ports());
            println!("Output Ports: {:?}", factory.get_output_ports());
        }
    }
}
