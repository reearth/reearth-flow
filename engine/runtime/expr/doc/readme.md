# Expression Language

## Literals

- integer: `42`
- float: `1.5`
- string: `"hello"`
- bool: `true`, `false`
- null: `null`
- array: `[1, "a", true]`
- map: `{"key": 1}` or `map([["key", val]])`

## Operators

### Precedence (high to low)

1. `map[key]` `list[start:stop:step]` `.method()`
2. unary `!` `-`
3. `*` `/`
4. `+` `-`
5. `<` `<=` `>` `>=`
6. `==` `!=` `in`
7. `&&`
8. `||`

## Built-ins

- casts: `str()` `int()` `float()` `bool()` `list()` `map()`
- Url: `.parent()` `.name()` `.stem()` `.extension()` `.__div__(rel)`
- string: `.len()` `.trim()` `.split(sep)` `.contains(s)` `.starts_with(s)` `.ends_with(s)` `.replace(from, to)`
- array: `.len()` `.contains(v)`
