# Creating IDs

The `uuid` module creates unique IDs. They are UUIDs, a standard ID format that looks like `2ed6657d-e927-568b-95e1-2665a8aea6a2`. Every ID is a lowercase string. The full definition is in [the spec](../spec/6-uuid-module.md).

| Expression      | Returns                                                         |
| --------------- | --------------------------------------------------------------- |
| `uuid.of(x)`    | An ID made from `x`. The same `x` always gives the same ID.     |
| `uuid.random()` | A new random ID on every call.                                  |

## Which one to use

Use **`uuid.of`** whenever the feature has something that identifies it, such as a building ID. Running the workflow again on the same data gives the same IDs, so output files stay identical between runs and IDs stay stable for anyone who links to them.

Use **`uuid.random`** only when nothing identifies the feature. Every run gives different IDs, and every feature gets its own, even when two features are otherwise identical.

## Giving buildings a PLATEAU gml:id

PLATEAU expects a `gml:id` made of the package name, an underscore, and a UUID, for example `bldg_2ed6657d-e927-568b-95e1-2665a8aea6a2`. In an Attribute Manager, create `gml:id` with:

```
"bldg_" + uuid.of(attributes["bldg_id"])
```

Each building gets an ID that depends only on its `bldg_id`, so the same building gets the same `gml:id` on every run.

**The value must be unique.** Features with the same value get the same ID, and the quality check reports duplicate `gml:id`s. If a value is not unique on its own, join it with something that is, for example `uuid.of(attributes["bldg_id"] + "/" + attributes["part_id"])`.

Strings and numbers both work. A number gives the same ID as the same number written as text, so `uuid.of(1)` and `uuid.of("1")` are equal. A missing value (`null`) is an error rather than an ID, because every feature missing the value would otherwise get the same ID.

## Matching IDs made elsewhere

`uuid.of(x)` makes standard version 5 UUIDs in a fixed namespace, listed in the spec. Another system can reproduce Flow's IDs from that namespace.

If you need to match IDs another system already makes from the same values, pass that system's namespace as a second argument:

```
uuid.of(attributes["code"], "6ba7b811-9dad-11d1-80b4-00c04fd430c8")
```

A namespace can also come from a workflow variable, such as `variables["ID_NAMESPACE"]`. Keep it fixed. Changing it changes every ID the workflow produces.
