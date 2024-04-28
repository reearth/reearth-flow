mod helper;

#[tokio::main]
async fn main() {
    let runner = helper::init_execute_runner("attribute_reader.yml");
    let result = runner.start().await;
    assert!(result.is_ok());
}
