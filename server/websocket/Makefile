.PHONY: all gen-thrift clean-thrift

all: gen-thrift

# Generate Thrift code
gen-thrift:
	@echo "Generating Rust Thrift code..."
	@mkdir -p src/thrift
	@rm -f src/document.rs
	@thrift -r --gen rs:thrift_crate=thrift -out src thrift/document.thrift
	@if [ -f src/document.rs ]; then \
		mv src/document.rs src/thrift/; \
	fi
	@echo "Thrift code generation completed."

# Clean generated Thrift code
clean-thrift:
	@echo "Cleaning up generated Thrift code..."
	@rm -f src/thrift/document.rs
	@echo "Cleanup completed." 