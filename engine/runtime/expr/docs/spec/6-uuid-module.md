# (DRAFT) UUID Module

The `uuid` module generates UUIDs as defined by [RFC 9562](https://www.rfc-editor.org/rfc/rfc9562).
Its names follow Python's `uuid` module.

UUIDs are `str` values in lowercase hyphenated form: 32 lowercase hexadecimal digits in groups of 8, 4, 4, 4 and 12, separated by `-` (e.g. `"2ed6657d-e927-568b-95e1-2665a8aea6a2"`).
The module has no separate UUID type, so a result can be concatenated, compared and stored like any other string, and a namespace is given as such a string.

## uuid.uuid5

`uuid.uuid5(namespace, name)` returns the name-based UUID (version 5, SHA-1) of `name`'s UTF-8 bytes within `namespace`.
The result depends only on its arguments: the same `namespace` and `name` always give the same UUID, across evaluations and across runs, and any RFC 9562 implementation reproduces it.
This is the function for identifiers derived from source data, since re-processing the same input reproduces the same identifiers.

`namespace` is a UUID as a `str` in hyphenated form, either case. Both arguments are required.

`name` is a `str`, `int` or `float`. A number gives the same UUID as its `str()` form, so `uuid.uuid5(ns, 1)` equals `uuid.uuid5(ns, "1")`.

## uuid.uuid4

`uuid.uuid4()` returns a random UUID (version 4).

Each call returns a new value. Evaluating the same expression again, such as once per feature, calls it again.

## uuid.NAMESPACE_FLOW

`uuid.NAMESPACE_FLOW` is the namespace `"8d8d96bb-df81-560f-bbd5-794fe22e23c0"`, for UUIDs derived with `uuid.uuid5` that need no other namespace.
It is the version 5 UUID of the URL `https://reearth.io/flow/uuid` in the RFC 9562 URL namespace, and does not change.
