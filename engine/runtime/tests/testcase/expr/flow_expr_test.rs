use crate::helper::execute_with_test_assert;

#[test]
fn test_url_path_join() {
    execute_with_test_assert("expr/flow_expr_test", "expect.json");
}
