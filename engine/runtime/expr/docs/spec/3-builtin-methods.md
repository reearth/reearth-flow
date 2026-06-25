# (DRAFT) Builtin Methods

Methods defined on the builtin types `str`, `list`, and `dict`.

## string methods

### `strip`

For string `s`, `s.strip()` returns a copy of `s` with leading and trailing whitespace removed.

### `lstrip`

For string `s`, `s.lstrip()` returns a copy of `s` with leading whitespace removed.

### `rstrip`

For string `s`, `s.rstrip()` returns a copy of `s` with trailing whitespace removed.

### `upper`

For string `s`, `s.upper()` returns a copy of `s` converted to uppercase.

### `lower`

For string `s`, `s.lower()` returns a copy of `s` converted to lowercase.

### `split`

For string `s`, `s.split()` splits `s` on any whitespace, returning a list of non-empty parts.

`s.split(sep)` splits `s` on `sep` from left to right, returning a list of parts.

`s.split(sep, limit)` splits at most `limit` times, producing at most `limit + 1` parts.

### `rsplit`

For string `s`, `s.rsplit(sep)` splits `s` on `sep`, returning a list of parts.

`s.rsplit(sep, limit)` splits at most `limit` times from right to left, producing at most `limit + 1` parts.

### `starts_with`

For string `s`, `s.starts_with(prefix)` returns `true` if `s` begins with `prefix`.

### `ends_with`

For string `s`, `s.ends_with(suffix)` returns `true` if `s` ends with `suffix`.

### `replace`

For string `s`, `s.replace(old, new)` returns a copy of `s` with all occurrences of `old` replaced by `new`.

### `remove_prefix`

For string `s`, `s.remove_prefix(prefix)` returns a copy of `s` with `prefix` removed from the start.
If `s` does not begin with `prefix`, returns `s` unchanged.

### `remove_suffix`

For string `s`, `s.remove_suffix(suffix)` returns a copy of `s` with `suffix` removed from the end.
If `s` does not end with `suffix`, returns `s` unchanged.

### `find`

For string `s`, `s.find(sub)` returns the codepoint index of the first occurrence of `sub` in `s`, or `null` if not found.

### `rfind`

For string `s`, `s.rfind(sub)` returns the codepoint index of the last occurrence of `sub` in `s`, or `null` if not found.

### `join`

For string `s`, `s.join(list)` returns a string formed by concatenating string elements of `list` with `s` as separator.

## list methods

### `append`

For list `l`, `l.append(v)` appends `v` to the end of `l`.

### `pop`

For list `l`, `l.pop()` removes and returns the last element of `l`.

`l.pop(i)` removes and returns the element at index `i`. Negative indices count from the end.

### `extend`

For list `l`, `l.extend(other)` appends all elements of `other` to `l`.

### `truncate`

For list `l`, `l.truncate(n)` removes all elements at index `n` and beyond. If `n` is greater than or equal to the length of `l`, `l` is unchanged.

### `get`

For list `l`, `l.get(i)` returns the element at index `i`, or `null` if out of bounds. Negative indices count from the end.

`l.get(i, default)` returns `default` instead of `null` when out of bounds.

### `index`

For list `l`, `l.index(v)` returns the index of the first element equal to `v`, or `null` if not found.

### `rindex`

For list `l`, `l.rindex(v)` returns the index of the last element equal to `v`, or `null` if not found.

## dict methods

### `get`

For dict `d`, `d.get(key)` returns the value for `key`, or `null` if the key is absent.

`d.get(key, default)` returns `default` instead of `null` when the key is absent.

### `pop`

For dict `d`, `d.pop(key)` removes `key` from `d` and returns its value, or `null` if the key is absent.

### `update`

For dict `d`, `d.update(other)` inserts or overwrites entries in `d` with all key-value pairs from `other`.

### `keys`

For dict `d`, `d.keys()` returns a list of all keys in `d`.

### `values`

For dict `d`, `d.values()` returns a list of all values in `d`.

### `items`

For dict `d`, `d.items()` returns a list of `[key, value]` pairs for each entry in `d`.
