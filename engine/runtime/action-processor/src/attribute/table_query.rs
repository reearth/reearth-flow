//! Query language for the source paths of the Attribute Table Extractor.
//!
//! A query is a sequence of segments separated by whitespace. Each segment is a key, optionally
//! followed by one or more selectors:
//!
//! ```text
//! query    := segment (WS+ segment)*
//! segment  := key selector*
//! key      := one or more characters other than whitespace, '[' and ']'
//! selector := "[*]" | "[0]"
//! ```
//!
//! A query is evaluated over a sequence of results, which starts as the feature's attributes:
//!
//! - key `k`: each result that is a map containing `k` is replaced by the value of `k`, as is,
//!   even when that value is a list. Every other result, including a list, is dropped.
//! - `[*]`: each result that is a list is replaced by its items. Every other result is kept.
//! - `[0]`: `[*]`, then only the first result is kept.
//!
//! A list is never searched implicitly: crossing a list takes `[*]` (all items) or `[0]` (the
//! first). Both treat a value that is not a list as a list of that one value, so a query works
//! the same whether the CityGML reader found an element once (a value) or several times (a list).

use std::fmt;

use reearth_flow_types::{AttributeValue, Attributes};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Query {
    /// The key of the first segment, looked up in the feature's attributes.
    root: String,
    /// The steps after the root key.
    steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq)]
enum Step {
    Key(String),
    All,
    First,
}

#[derive(Debug, Clone, PartialEq)]
pub(super) struct ParseError {
    query: String,
    reason: &'static str,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "invalid query `{}`: {}", self.query, self.reason)
    }
}

impl std::error::Error for ParseError {}

impl Query {
    pub(super) fn parse(query: &str) -> Result<Self, ParseError> {
        let error = |reason| ParseError {
            query: query.to_string(),
            reason,
        };
        let mut steps = Vec::new();
        for segment in query.split_whitespace() {
            let key_end = segment.find(['[', ']']).unwrap_or(segment.len());
            let (key, mut selectors) = segment.split_at(key_end);
            if key.is_empty() {
                return Err(error("a segment must start with a key"));
            }
            steps.push(Step::Key(key.to_string()));
            while !selectors.is_empty() {
                let step = if let Some(rest) = selectors.strip_prefix("[*]") {
                    selectors = rest;
                    Step::All
                } else if let Some(rest) = selectors.strip_prefix("[0]") {
                    selectors = rest;
                    Step::First
                } else {
                    return Err(error("a selector must be `[*]` or `[0]`"));
                };
                steps.push(step);
            }
        }
        let mut steps = steps.into_iter();
        match steps.next() {
            Some(Step::Key(root)) => Ok(Self {
                root,
                steps: steps.collect(),
            }),
            _ => Err(error("a query must have at least one segment")),
        }
    }

    /// Every result of the query, in document order.
    pub(super) fn select<'a>(&self, attributes: &'a Attributes) -> Vec<&'a AttributeValue> {
        let mut results: Vec<&AttributeValue> =
            attributes.get(self.root.as_str()).into_iter().collect();
        for step in &self.steps {
            results = match step {
                Step::Key(key) => results
                    .into_iter()
                    .filter_map(|value| match value {
                        AttributeValue::Map(map) => map.get(key.as_str()),
                        _ => None,
                    })
                    .collect(),
                Step::All => results.into_iter().flat_map(items).collect(),
                Step::First => results.into_iter().flat_map(items).take(1).collect(),
            };
        }
        results
    }

    /// The query's value: `None` without results, the result itself when there is exactly one,
    /// and a list of the results otherwise.
    pub(super) fn evaluate(&self, attributes: &Attributes) -> Option<AttributeValue> {
        let mut results = self.select(attributes);
        match results.len() {
            0 => None,
            1 => results.pop().cloned(),
            _ => Some(AttributeValue::Array(
                results.into_iter().cloned().collect(),
            )),
        }
    }
}

/// The items of a list, or the value itself when it is not a list.
fn items(value: &AttributeValue) -> Vec<&AttributeValue> {
    match value {
        AttributeValue::Array(list) => list.iter().collect(),
        other => vec![other],
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::{Value, json};

    use super::*;

    /// Runs `query` over the attributes `data` and returns its results as a JSON list.
    fn select(data: Value, query: &str) -> Value {
        let AttributeValue::Map(attributes) = AttributeValue::from(data) else {
            panic!("test data must be an object");
        };
        let query = Query::parse(query).unwrap();
        Value::Array(
            query
                .select(&attributes)
                .into_iter()
                .cloned()
                .map(Value::from)
                .collect(),
        )
    }

    /// Same as `select`, through `evaluate`; `null` stands for no value.
    fn evaluate(data: Value, query: &str) -> Value {
        let AttributeValue::Map(attributes) = AttributeValue::from(data) else {
            panic!("test data must be an object");
        };
        Query::parse(query)
            .unwrap()
            .evaluate(&attributes)
            .map_or(Value::Null, Value::from)
    }

    // Key

    #[test]
    fn key_takes_the_value_of_a_map_entry_as_is() {
        let data = json!({"a": {"b": 1, "l": [1, 2]}});
        assert_eq!(select(data.clone(), "a"), json!([{"b": 1, "l": [1, 2]}]));
        assert_eq!(select(data.clone(), "a b"), json!([1]));
        assert_eq!(select(data, "a l"), json!([[1, 2]]));
    }

    #[test]
    fn key_drops_results_without_it() {
        let data = json!({"a": {"b": 1}, "s": "text"});
        assert_eq!(select(data.clone(), "missing"), json!([]));
        assert_eq!(select(data.clone(), "a missing"), json!([]));
        assert_eq!(select(data, "s b"), json!([]));
    }

    #[test]
    fn key_does_not_look_into_a_list() {
        let data = json!({"a": [{"b": 1}, {"b": 2}]});
        assert_eq!(select(data, "a b"), json!([]));
    }

    // [*]

    #[test]
    fn all_replaces_a_list_by_its_items() {
        let data = json!({"a": [{"b": 1}, {"b": 2}], "e": []});
        assert_eq!(select(data.clone(), "a[*]"), json!([{"b": 1}, {"b": 2}]));
        assert_eq!(select(data.clone(), "a[*] b"), json!([1, 2]));
        assert_eq!(select(data, "e[*]"), json!([]));
    }

    #[test]
    fn all_keeps_a_value_that_is_not_a_list() {
        let data = json!({"a": {"b": 1}});
        assert_eq!(select(data.clone(), "a[*]"), json!([{"b": 1}]));
        assert_eq!(select(data, "a[*] b"), json!([1]));
    }

    #[test]
    fn all_expands_one_level_only() {
        let data = json!({"a": [[1, 2], 3]});
        assert_eq!(select(data.clone(), "a[*]"), json!([[1, 2], 3]));
        assert_eq!(select(data, "a[*][*]"), json!([1, 2, 3]));
    }

    #[test]
    fn key_after_all_drops_items_without_it() {
        // A CityGML 3 ADE hook holds one class per item: a key selects that class.
        let data = json!({"ade": [{"uro:Detail": {"x": 1}}, {"uro:ID": {"id": "a"}}]});
        assert_eq!(select(data, "ade[*] uro:ID id"), json!(["a"]));
    }

    // [0]

    #[test]
    fn first_takes_the_first_item_of_a_list() {
        let data = json!({"a": [1, 2], "e": []});
        assert_eq!(select(data.clone(), "a[0]"), json!([1]));
        assert_eq!(select(data, "e[0]"), json!([]));
    }

    #[test]
    fn first_keeps_a_value_that_is_not_a_list() {
        let data = json!({"a": {"b": 1}, "s": "text"});
        assert_eq!(select(data.clone(), "a[0] b"), json!([1]));
        assert_eq!(select(data, "s[0]"), json!(["text"]));
    }

    #[test]
    fn first_applies_to_all_results_not_to_each() {
        let data = json!({"a": [{"b": [1, 2]}, {"b": [3, 4]}]});
        assert_eq!(select(data.clone(), "a[*] b"), json!([[1, 2], [3, 4]]));
        assert_eq!(select(data, "a[*] b[0]"), json!([1]));
    }

    // Evaluation

    #[test]
    fn evaluate_gives_nothing_one_value_or_a_list() {
        let data = json!({"a": [{"b": 1}, {"b": 2}, {"c": 3}]});
        assert_eq!(evaluate(data.clone(), "a[*] missing"), Value::Null);
        assert_eq!(evaluate(data.clone(), "a[*] c"), json!(3));
        assert_eq!(evaluate(data, "a[*] b"), json!([1, 2]));
    }

    #[test]
    fn a_query_gives_the_same_value_for_one_element_and_for_a_list_of_one() {
        // The CityGML reader gives one element as its value and several as a list.
        let one = json!({"ade": {"uro:ID": {"id": "a"}}});
        let many = json!({"ade": [{"uro:ID": {"id": "a"}}, {"uro:ID": {"id": "b"}}]});
        assert_eq!(evaluate(one.clone(), "ade[*] uro:ID[0] id"), json!("a"));
        assert_eq!(evaluate(many.clone(), "ade[*] uro:ID[0] id"), json!("a"));
        assert_eq!(evaluate(one, "ade[*] uro:ID id"), json!("a"));
        assert_eq!(evaluate(many, "ade[*] uro:ID id"), json!(["a", "b"]));
    }

    // Syntax

    #[test]
    fn segments_are_separated_by_any_whitespace() {
        let data = json!({"a": {"b": [1, 2]}});
        assert_eq!(select(data, "  a \t b[*][0]\n"), json!([1]));
    }

    #[test]
    fn rejects_malformed_queries() {
        for query in [
            "", "   ", "[0]", "a [0]", "a[1]", "a[-1]", "a[", "a[*", "a]", "a[0]b",
        ] {
            assert!(Query::parse(query).is_err(), "{query:?} should be rejected");
        }
    }
}
