//! Query language for the source paths of the Attribute Table Extractor.
//!
//! A query is a sequence of steps separated by whitespace. It is evaluated over a sequence of
//! results; the first step must be a key, looked up in the feature's attributes.
//!
//! - `k`: per result; the value of `k` if the result is a map that has it, otherwise nothing
//! - `[]`: per result; a list stays as is, anything else (`null` included) becomes a one-item list
//! - `[*]`: per result; a list becomes its items, anything else gives nothing
//! - `[n]`: per result; item `n` of a list (negative counts from the end), anything else or out of
//!   range gives nothing
//! - `[k]`: over all results; one list of their `k` values as is (lists stay nested), results
//!   without `k` skipped, may be `[]`
//! - output: zero results give nothing, one result gives that value, several results give nothing
//!
//! Inside brackets, an integer is an index, `*` expands, and anything else is a key to gather.

use std::borrow::Cow;
use std::fmt;

use reearth_flow_types::{AttributeValue, Attributes};

#[derive(Debug, Clone, PartialEq)]
pub(super) struct Query {
    /// The first step's key, looked up in the feature's attributes.
    root: String,
    steps: Vec<Step>,
}

#[derive(Debug, Clone, PartialEq)]
enum Step {
    Key(String),
    Wrap,
    Expand,
    Index(i64),
    Gather(String),
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

type Results<'a> = Vec<Cow<'a, AttributeValue>>;

impl Query {
    pub(super) fn parse(query: &str) -> Result<Self, ParseError> {
        let error = |reason| ParseError {
            query: query.to_string(),
            reason,
        };
        let mut steps = query.split_whitespace().map(|token| {
            if let Some(inner) = token.strip_prefix('[').and_then(|t| t.strip_suffix(']')) {
                if inner.contains(['[', ']']) {
                    Err(error("brackets cannot be nested"))
                } else if inner.is_empty() {
                    Ok(Step::Wrap)
                } else if inner == "*" {
                    Ok(Step::Expand)
                } else if let Ok(n) = inner.parse() {
                    Ok(Step::Index(n))
                } else {
                    Ok(Step::Gather(inner.to_string()))
                }
            } else if token.contains(['[', ']']) {
                Err(error(
                    "a bracket step must be separated from a key by whitespace",
                ))
            } else {
                Ok(Step::Key(token.to_string()))
            }
        });
        let root = match steps.next() {
            Some(Ok(Step::Key(root))) => root,
            Some(Err(e)) => return Err(e),
            Some(Ok(_)) => return Err(error("a query must start with a key")),
            None => return Err(error("a query must have at least one step")),
        };
        Ok(Self {
            root,
            steps: steps.collect::<Result<_, _>>()?,
        })
    }

    /// Every result of the query, in order.
    fn select<'a>(&self, attributes: &'a Attributes) -> Results<'a> {
        let mut results: Results<'a> = attributes
            .get(self.root.as_str())
            .map(Cow::Borrowed)
            .into_iter()
            .collect();
        for step in &self.steps {
            results = match step {
                Step::Key(key) => results.into_iter().filter_map(|r| get(r, key)).collect(),
                Step::Wrap => results.into_iter().map(wrap).collect(),
                Step::Expand => results.into_iter().flat_map(expand).collect(),
                Step::Index(n) => results.into_iter().filter_map(|r| index(r, *n)).collect(),
                Step::Gather(key) => vec![Cow::Owned(AttributeValue::Array(
                    results
                        .into_iter()
                        .filter_map(|r| get(r, key))
                        .map(Cow::into_owned)
                        .collect(),
                ))],
            };
        }
        results
    }

    /// The query's value: the result when there is exactly one, otherwise nothing.
    pub(super) fn evaluate(&self, attributes: &Attributes) -> Option<AttributeValue> {
        let mut results = self.select(attributes);
        if results.len() == 1 {
            results.pop().map(Cow::into_owned)
        } else {
            None
        }
    }
}

fn get<'a>(value: Cow<'a, AttributeValue>, key: &str) -> Option<Cow<'a, AttributeValue>> {
    match value {
        Cow::Borrowed(AttributeValue::Map(map)) => map.get(key).map(Cow::Borrowed),
        Cow::Owned(AttributeValue::Map(mut map)) => map.swap_remove(key).map(Cow::Owned),
        _ => None,
    }
}

fn wrap(value: Cow<'_, AttributeValue>) -> Cow<'_, AttributeValue> {
    match value.as_ref() {
        AttributeValue::Array(_) => value,
        _ => Cow::Owned(AttributeValue::Array(vec![value.into_owned()])),
    }
}

fn expand(value: Cow<'_, AttributeValue>) -> Results<'_> {
    match value {
        Cow::Borrowed(AttributeValue::Array(list)) => list.iter().map(Cow::Borrowed).collect(),
        Cow::Owned(AttributeValue::Array(list)) => list.into_iter().map(Cow::Owned).collect(),
        _ => Vec::new(),
    }
}

fn index(value: Cow<'_, AttributeValue>, n: i64) -> Option<Cow<'_, AttributeValue>> {
    let len = match value.as_ref() {
        AttributeValue::Array(list) => list.len(),
        _ => return None,
    };
    let pos = if n >= 0 {
        usize::try_from(n).ok()?
    } else {
        len.checked_sub(usize::try_from(n.unsigned_abs()).ok()?)?
    };
    if pos >= len {
        return None;
    }
    match value {
        Cow::Borrowed(AttributeValue::Array(list)) => Some(Cow::Borrowed(&list[pos])),
        Cow::Owned(AttributeValue::Array(mut list)) => Some(Cow::Owned(list.swap_remove(pos))),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use serde_json::{Value, json};

    use super::*;

    fn attributes(data: Value) -> Attributes {
        let AttributeValue::Map(attributes) = AttributeValue::from(data) else {
            panic!("test data must be an object");
        };
        attributes
    }

    /// The results of `query` over `data`, as a JSON list.
    fn results(data: Value, query: &str) -> Value {
        let attributes = attributes(data);
        let results = Query::parse(query).unwrap().select(&attributes);
        Value::Array(results.into_iter().map(|r| r.into_owned().into()).collect())
    }

    /// The output of `query` over `data`; `null` stands for nothing.
    fn evaluate(data: Value, query: &str) -> Value {
        Query::parse(query)
            .unwrap()
            .evaluate(&attributes(data))
            .map_or(Value::Null, Value::from)
    }

    #[test]
    fn key() {
        let data = json!({"a": {"b": 1, "l": [1, 2]}, "s": "x"});
        assert_eq!(results(data.clone(), "a b"), json!([1]));
        assert_eq!(results(data.clone(), "a l"), json!([[1, 2]]));
        assert_eq!(results(data.clone(), "a missing"), json!([]));
        assert_eq!(results(data.clone(), "s b"), json!([]));
        assert_eq!(results(json!({"a": [{"b": 1}]}), "a b"), json!([]));
    }

    #[test]
    fn wrap() {
        let data = json!({"l": [1, 2], "m": {"b": 1}, "n": null});
        assert_eq!(results(data.clone(), "l []"), json!([[1, 2]]));
        assert_eq!(results(data.clone(), "m []"), json!([[{"b": 1}]]));
        assert_eq!(results(data, "n []"), json!([[null]]));
    }

    #[test]
    fn expand() {
        let data = json!({"l": [1, [2, 3]], "e": [], "m": {"b": 1}});
        assert_eq!(results(data.clone(), "l [*]"), json!([1, [2, 3]]));
        assert_eq!(results(data.clone(), "e [*]"), json!([]));
        assert_eq!(results(data, "m [*]"), json!([]));
    }

    #[test]
    fn index() {
        let data = json!({"l": [1, 2, 3], "m": {"b": 1}});
        assert_eq!(results(data.clone(), "l [0]"), json!([1]));
        assert_eq!(results(data.clone(), "l [-1]"), json!([3]));
        assert_eq!(results(data.clone(), "l [3]"), json!([]));
        assert_eq!(results(data.clone(), "l [-4]"), json!([]));
        assert_eq!(results(data, "m [0]"), json!([]));
        // per result: each list is indexed on its own, a missing item is dropped
        assert_eq!(
            results(json!({"l": [[1, 2], [3]]}), "l [*] [1]"),
            json!([2])
        );
    }

    #[test]
    fn gather() {
        let data = json!({"l": [{"k": [1, 2]}, {"x": 0}, 5, {"k": 3}]});
        assert_eq!(results(data.clone(), "l [*] [k]"), json!([[[1, 2], 3]]));
        assert_eq!(results(data.clone(), "l [*] [missing]"), json!([[]]));
        assert_eq!(results(json!({"m": {"k": 1}}), "m [k]"), json!([[1]]));
    }

    #[test]
    fn output() {
        let data = json!({"l": [1, 2], "one": [1]});
        assert_eq!(evaluate(data.clone(), "missing"), Value::Null);
        assert_eq!(evaluate(data.clone(), "one [*]"), json!(1));
        assert_eq!(evaluate(data.clone(), "l"), json!([1, 2]));
        assert_eq!(evaluate(data, "l [*]"), Value::Null);
    }

    #[test]
    fn one_query_for_every_shape_of_a_repeatable_element() {
        // The CityGML reader gives an element found once as its value and several as a list.
        let query = "a [] [*] [I] [0] id";
        let once = json!({"a": {"I": {"id": "x"}}});
        let mixed = json!({"a": [{"D": {"y": 1}}, {"I": {"id": "x"}}]});
        let duplicated = json!({"a": [{"I": {"id": "x"}}, {"I": {"id": "Null"}}]});
        assert_eq!(evaluate(once, query), json!("x"));
        assert_eq!(evaluate(mixed, query), json!("x"));
        assert_eq!(evaluate(duplicated, query), json!("x"));
    }

    #[test]
    fn syntax() {
        assert_eq!(results(json!({"a": [[1]]}), " a\t[*]\n [0] "), json!([1]));
        for query in ["", " ", "[0]", "[k] a", "a[0]", "a [0", "a 0]", "a [[0]]"] {
            assert!(Query::parse(query).is_err(), "{query:?} should be rejected");
        }
    }
}
