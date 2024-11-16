use crate::helper::execute_with_test_assert;

#[test]
fn test_attribute_python() {
    execute_with_test_assert("wasm/runtime_executor/attribute_python", "expect.json");
}
