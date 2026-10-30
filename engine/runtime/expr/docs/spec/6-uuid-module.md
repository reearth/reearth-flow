# (DRAFT) UUID Module

The `uuid` module generates UUIDs as defined by [RFC 9562](https://www.rfc-editor.org/rfc/rfc9562).

UUIDs are `str` values in lowercase hyphenated form: 32 lowercase hexadecimal digits in groups of 8, 4, 4, 4 and 12, separated by `-` (e.g. `"2ed6657d-e927-568b-95e1-2665a8aea6a2"`).
The module has no separate UUID type, so a result can be concatenated, compared and stored like any other string.

## uuid.v5

`uuid.v5(namespace, name)` returns the name-based UUID (version 5, SHA-1) of the `str` `name` within `namespace`.
`name` is hashed as its UTF-8 bytes.

`namespace` is a UUID given as a `str` in hyphenated form, either case.

The result depends only on `namespace` and `name`: the same arguments always give the same UUID, across evaluations and across runs.
This is the function for identifiers derived from source data, since re-processing the same input reproduces the same identifiers.

## uuid.v4

`uuid.v4()` returns a random UUID (version 4).

Each call returns a new value. Evaluating the same expression again, such as once per feature, calls it again.

## uuid.namespace_dns

The namespace UUID for fully qualified domain names, `"6ba7b810-9dad-11d1-80b4-00c04fd430c8"` ([RFC 9562 section 6.6](https://www.rfc-editor.org/rfc/rfc9562#section-6.6)).

## uuid.namespace_url

The namespace UUID for URLs, `"6ba7b811-9dad-11d1-80b4-00c04fd430c8"` ([RFC 9562 section 6.6](https://www.rfc-editor.org/rfc/rfc9562#section-6.6)).

## uuid.namespace_oid

The namespace UUID for ISO object identifiers, `"6ba7b812-9dad-11d1-80b4-00c04fd430c8"` ([RFC 9562 section 6.6](https://www.rfc-editor.org/rfc/rfc9562#section-6.6)).

## uuid.namespace_x500

The namespace UUID for X.500 distinguished names, `"6ba7b814-9dad-11d1-80b4-00c04fd430c8"` ([RFC 9562 section 6.6](https://www.rfc-editor.org/rfc/rfc9562#section-6.6)).
