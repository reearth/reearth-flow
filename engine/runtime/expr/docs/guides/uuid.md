# Generating UUIDs

The `uuid` module makes UUIDs: unique identifiers such as `2ed6657d-e927-568b-95e1-2665a8aea6a2`. Every function returns a lowercase, hyphenated string. The full definition is in [the spec](../spec/6-uuid-module.md).

| Expression                   | Returns                                                       |
| ---------------------------- | ------------------------------------------------------------- |
| `uuid.v5(namespace, name)`   | The same UUID every time for the same `namespace` and `name`. |
| `uuid.v4()`                  | A new random UUID on every call.                              |
| `uuid.namespace_url` etc.    | A standard namespace for `uuid.v5`.                           |

## Which one to use

Use **`uuid.v5`** whenever the feature has something that identifies it, such as a building ID. Running the workflow again on the same data gives the same UUIDs, so output files stay identical between runs and IDs stay stable for anyone who links to them.

Use **`uuid.v4`** only when nothing identifies the feature. Every run gives different UUIDs, and every feature gets its own, even when two features are otherwise identical.

## Giving buildings a PLATEAU gml:id

PLATEAU expects a `gml:id` made of the package name, an underscore, and a lowercase hyphenated UUID, for example `bldg_2ed6657d-e927-568b-95e1-2665a8aea6a2`. In an Attribute Manager, create `gml:id` with:

```
"bldg_" + uuid.v5(uuid.namespace_url, attributes["bldg_id"])
```

Each building gets a UUID that depends only on its `bldg_id`, so the same building gets the same `gml:id` on every run.

Two things to check:

- **The name must be unique.** Features with the same name get the same UUID, and the quality check reports duplicate `gml:id`s. If a value is not unique on its own, join it with something that is, for example `attributes["bldg_id"] + "/" + attributes["part_id"]`.
- **The name must be a string.** If the source attribute is a number, convert it first: `uuid.v5(uuid.namespace_url, str(attributes["bldg_id"]))`.

## Choosing a namespace

The namespace keeps UUIDs from different sources apart: the same name in two namespaces gives two unrelated UUIDs.

The built-in namespaces are the standard ones from RFC 9562:

| Constant              | Intended for                          |
| --------------------- | ------------------------------------- |
| `uuid.namespace_dns`  | Domain names, such as `example.com`   |
| `uuid.namespace_url`  | URLs                                  |
| `uuid.namespace_oid`  | ISO object identifiers                |
| `uuid.namespace_x500` | X.500 distinguished names             |

Any of them works for IDs from your own data. If two different workflows could produce the same names for different things, give each its own namespace: generate a UUID once (for example with `uuid.v4()`), store it as a workflow variable, and pass it in:

```
"bldg_" + uuid.v5(variables["ID_NAMESPACE"], attributes["bldg_id"])
```

Keep the variable's value fixed. Changing it changes every UUID the workflow produces.
