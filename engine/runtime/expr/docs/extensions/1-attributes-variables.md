# (DRAFT) `attributes` and `variables`

reearth-flow injects two global objects into expressions,
`attributes` and `variables`.

## `attributes`

Bound to the current feature's attributes.

Not bound when there is no current feature, referencing `attributes` in that context is an error.

### `attributes[key]`

Returns the value of the attribute named `key`.
Errors if `key` is absent.

### `attributes.get(key)`

Returns the value of the attribute named `key`, or `null` if `key` is absent.

`attributes.get(key, default)` returns `default` instead of `null` when `key` is absent.

### `key in attributes` / `key not in attributes`

Returns whether `key` names an attribute present on the current feature.

### `for key in attributes`

Iterates the names of the feature's attributes.

## `variables`

Bound to the workflow's variables — the entries declared under `with:`.

Unlike `attributes`, always bound, even when there is no current feature.

### `variables[key]`

Returns the value of the workflow variable named `key`. Errors if `key` is absent.

### `variables.get(key)`

Returns the value of the workflow variable named `key`, or `null` if `key` is absent.

`variables.get(key, default)` returns `default` instead of `null` when `key` is absent.
