//! Lookups in the compiled XSDs of the application schema.

use std::collections::HashSet;

use fastxml::schema::types::{CompiledSchema, NsName};

/// The substitution group heads of the global element `namespace`:`local`,
/// from its own head up, excluding the element itself.
pub(super) fn substitution_heads<'a>(
    schema: &'a CompiledSchema,
    namespace: &str,
    local: &str,
) -> impl Iterator<Item = &'a NsName> {
    let mut current = schema.element_ns(namespace, local);
    let mut visited = HashSet::new();
    std::iter::from_fn(move || {
        let head = current?.substitution_ns.as_ref()?;
        // A cyclic substitution group of a broken schema ends at its first
        // repeated head.
        if !visited.insert(head) {
            return None;
        }
        current = schema.element_ns(&head.namespace_uri, &head.local_name);
        Some(head)
    })
}

#[cfg(test)]
mod tests {
    use fastxml::schema::types::ElementDef;
    use pretty_assertions::assert_eq;

    use super::*;

    const NS: &str = "http://example.com/test";

    /// A schema whose element `name` substitutes for `head`, for each pair.
    fn schema(substitutions: &[(&str, &str)]) -> CompiledSchema {
        let mut schema = CompiledSchema::new();
        for (name, head) in substitutions {
            let mut element = ElementDef::new(*name);
            element.substitution_ns = Some(NsName::new(NS, *head));
            schema.elements_ns.insert(NsName::new(NS, *name), element);
        }
        schema
    }

    fn heads(schema: &CompiledSchema, local: &str) -> Vec<String> {
        substitution_heads(schema, NS, local)
            .map(|head| head.local_name.to_string())
            .collect()
    }

    #[test]
    fn test_heads_up_to_the_root() {
        let schema = schema(&[
            ("Polygon", "AbstractSurface"),
            ("AbstractSurface", "AbstractGeometry"),
        ]);
        assert_eq!(
            heads(&schema, "Polygon"),
            ["AbstractSurface", "AbstractGeometry"]
        );
    }

    #[test]
    fn test_cyclic_substitution_group_ends() {
        let schema = schema(&[("A", "B"), ("B", "C"), ("C", "B")]);
        assert_eq!(heads(&schema, "A"), ["B", "C"]);
    }
}
