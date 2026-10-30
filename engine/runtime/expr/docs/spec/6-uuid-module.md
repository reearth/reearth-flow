# (DRAFT) UUID Module

The `uuid` module generates UUIDs as defined by [RFC 9562](https://www.rfc-editor.org/rfc/rfc9562).

UUIDs are `str` values in lowercase hyphenated form: 32 lowercase hexadecimal digits in groups of 8, 4, 4, 4 and 12, separated by `-` (e.g. `"2ed6657d-e927-568b-95e1-2665a8aea6a2"`).
The module has no separate UUID type, so a result can be concatenated, compared and stored like any other string.

## uuid.of

`uuid.of(value)` returns a UUID derived from `value`. The result depends only on `value`: the same `value` always gives the same UUID, across evaluations and across runs.
This is the function for identifiers derived from source data, since re-processing the same input reproduces the same identifiers.

`value` is a `str`, `int` or `float`. A number gives the same UUID as its `str()` form, so `uuid.of(1)` equals `uuid.of("1")`.

The result is the name-based UUID (version 5, SHA-1) of `value`'s UTF-8 bytes within the namespace `8d8d96bb-df81-560f-bbd5-794fe22e23c0`, so it can be reproduced by any RFC 9562 implementation.
That namespace is itself the version 5 UUID of the URL `https://reearth.io/flow/uuid` in the RFC 9562 URL namespace, and does not change.

`uuid.of(value, namespace)` uses `namespace` instead, a UUID given as a `str` in hyphenated form, either case.
This produces the same UUIDs as another system that derives version 5 UUIDs from the same names in that namespace.

## uuid.random

`uuid.random()` returns a random UUID (version 4).

Each call returns a new value. Evaluating the same expression again, such as once per feature, calls it again.
