## Usage
### Command Line
* To run a flow, use the following command:
``` sh
## Specify the workflow yaml you want to run
export FLOW_EXAMPLE_TARGET_WORKFLOW=fixture/workflow/requirement/plateau4/a002-2/workflow.yml
cargo run --package reearth-flow-examples --example example_main
```

### SQL

#### SQLite
``` sql
CREATE TABLE features
  (
      attribute_text_01 TEXT    NOT NULL,
      attribute_int_01 INTEGER    NOT NULL
  );
insert into features(attribute_text_01, attribute_int_01) values ('test01-01', 1),('test02-01', 2);
```
