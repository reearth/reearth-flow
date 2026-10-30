# Creating IDs

The `uuid` module creates unique IDs. They are UUIDs, a standard ID format that looks like `2ed6657d-e927-568b-95e1-2665a8aea6a2`. Every ID is a lowercase string. The module works like Python's `uuid` module, and the full definition is in [the spec](../spec/6-uuid-module.md).

| Expression                             | Returns                                                         |
| -------------------------------------- | --------------------------------------------------------------- |
| `uuid.uuid5(uuid.NAMESPACE_FLOW, x)`   | An ID made from `x`. The same `x` always gives the same ID.     |
| `uuid.uuid4()`                         | A new random ID on every call. It changes every run.            |

## Which one to use

Use **`uuid.uuid5`** whenever the feature has something that identifies it, such as a building ID. Running the workflow again on the same data gives the same IDs, so output files stay identical between runs and IDs stay stable for anyone who links to them.

Use **`uuid.uuid4`** only when nothing identifies the feature. Every run gives different IDs, and every feature gets its own, even when two features are otherwise identical.

## Giving buildings a PLATEAU gml:id

PLATEAU expects a `gml:id` made of the package name, an underscore, and a UUID, for example `bldg_2ed6657d-e927-568b-95e1-2665a8aea6a2`. In an Attribute Manager, create `gml:id` with:

```
"bldg_" + uuid.uuid5(uuid.NAMESPACE_FLOW, attributes["bldg_id"])
```

Each building gets an ID that depends only on its `bldg_id`, so the same building gets the same `gml:id` on every run.

**The value must be unique.** Features with the same value get the same ID, and the quality check reports duplicate `gml:id`s. If a value is not unique on its own, join it with something that is, for example `uuid.uuid5(uuid.NAMESPACE_FLOW, attributes["bldg_id"] + "/" + attributes["part_id"])`.

The value must be a string. If it is a number, convert it with `str()`: `uuid.uuid5(uuid.NAMESPACE_FLOW, str(attributes["code"]))`. Watch for whole numbers stored as decimals: `str(1.0)` is `"1.0"`, not `"1"`, so it gives a different ID. A missing value (`null`) is an error rather than an ID, because every feature missing the value would otherwise get the same ID.

## The namespace

The first argument of `uuid.uuid5` is a namespace: another UUID that the ID is made within. The same value gives different IDs in different namespaces. `uuid.NAMESPACE_FLOW` is Flow's own namespace, and suits any ID that doesn't need to match another system.

To match IDs another system already makes from the same values, pass that system's namespace instead:

```
uuid.uuid5("6ba7b811-9dad-11d1-80b4-00c04fd430c8", attributes["code"])
```

A namespace can also come from a workflow variable, such as `variables["ID_NAMESPACE"]`. Keep it fixed. Changing it changes every ID the workflow produces.
