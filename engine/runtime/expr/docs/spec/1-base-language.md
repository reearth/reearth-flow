# Base Language

**FlowExpr** is an expression language: every program is an expression that evaluates to a single value.

This document specifies the base language: basic types and operators, and the semantics.

## null type

`null` represents the absence of a value. There is exactly one null value, written `null`.

## boolean type

A boolean value, written `true` or `false`.

## integer type

A 64-bit signed integer. Integer literals match one of:

- decimal: `[0-9]+`
- binary: `0[bB][01]+`
- octal: `0[oO][0-7]+`
- hexadecimal: `0[xX][0-9a-fA-F]+`

### constructor

`int` is a type object. Calling `int(value)` constructs an integer based on the type of `value`:

- `int`: same value
- `float`: truncated toward zero; error if not finite or out of range
- `bool`: `0` or `1`
- `string`: decimal integer parsed from the trimmed string; error if not parseable
- no argument: `0`

## float type

A 64-bit IEEE 754 double-precision floating-point number.
Float literals must contain a decimal point or an exponent (e.g. `1.0`, `1e10`).

### constructor

`float` is a type object. Calling `float(value)` constructs a float based on the type of `value`:

- `float`: same value
- `int`: same numeric value
- `bool`: `1.0` or `0.0`
- `string`: decimal float parsed from the trimmed string; error if not parseable
- no argument: `0.0`

## string type

An immutable sequence of Unicode codepoints.
String literals are enclosed in double quotes (e.g. `"hello"`).

### constructor

`str` is a type object. Calling `str(value)` constructs a string based on the type of `value`:

- `string`: same value
- `int`: decimal representation
- `float`: a decimal representation; the exact format is unspecified
- `bool`: `"true"` or `"false"`
- `null`: `"null"`
- no argument: `""`

## list type

A mutable, ordered sequence of values.
List literals use square brackets (e.g. `[1, "two", true]`). Elements may be of any type.

### constructor

`list` is a type object. Calling `list(value)` constructs a list based on the type of `value`:

- `list`: shallow copy
- `string`: list of single-character strings, one per Unicode codepoint
- `dict`: list of keys
- no argument: `[]`

## dictionary type

A mutable, ordered map from string keys to values. Other key types are undefined.
Dict literals use curly braces (e.g. `{"a": 1, "b": 2}`).
Insertion order preservation is not stabilized by this spec.

### constructor

`dict` is a type object. Calling `dict(value)` constructs a dictionary based on the type of `value`:

- `dict`: shallow copy
- `list`: each element is a 2-element `[key, value]` list where `key` is a string
- no argument: `{}`

## operators

Operators listed from lowest to highest precedence.
`left` and `right` denote associativity; `prefix` denotes a unary operator with no left operand.

- `or` (left)
- `and` (left)
- `not` (prefix)
- `==` `!=` `in` `not in` (left)
- `<` `<=` `>` `>=` (left)
- `|` (left)
- `^` (left)
- `&` (left)
- `<<` `>>` (left)
- `+` `-` (left)
- `*` `/` `//` `%` (left)
- unary `-` (prefix)
- `**` (right)

## integer and float division

`/` always returns a float, even when both operands are integers.
`//` is floor division: the result is rounded toward negative infinity.
`%` remainder has the same sign as the divisor.

Division by zero is an error for `/`, `//`, and `%`.

## bitwise operators

`&`, `|`, `^`, `<<`, `>>` operate on integers. Both operands must be non-negative integers.

Implementations must support left shift results across the full range of non-negative integers.
Right shift (`>>`) with an amount >= 63 returns `0`.

## mutability

All values have reference semantics: assigning a value to another variable does not copy it.
Mutations through any alias are visible through all of them.

Whether a type is effectively mutable depends solely on whether it exposes any mutating operations.

## cyclic reference

Cyclic references (e.g. a list that contains itself) are undefined behavior.

Notes: An implication is that FlowExpr may be implemented with reference counting memory management.
However, silent memory leak is usually unwanted in most embeded evaluation environments that need to be handled or at least detected.