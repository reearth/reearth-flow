use crate::helper::execute;

#[test]
fn test_url_path_join() {
    let tempdir = execute("expr/flow_expr_test", vec![]).unwrap();
    let output = std::fs::read_to_string(tempdir.path().join("result.json"))
        .expect("result.json not created");
    assert!(
        output.contains("\"result\":\"file:///tmp/hello.txt\""),
        "expected file:///tmp/hello.txt, got: {output}"
    );
}
