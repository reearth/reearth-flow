# (DRAFT) Base Language

**FlowExpr** is an expression language: every program evaluates to a single value.

This document specifies the base language: basic types and operators, and the semantics.

## null type

`null` represents the absence of a value. There is exactly one null value, written `null`.

## boolean type

A boolean value, written `true` or `false`.

### truthiness

Truthiness by type:

- `null`: always falsy
- `bool`: its value
- `int`: falsy if `0`, truthy otherwise
- `float`: falsy if `0.0`, truthy otherwise
- `str`: falsy if empty, truthy otherwise
- `list`: falsy if empty, truthy otherwise
- `dict`: falsy if empty, truthy otherwise

### constructor

`bool` is a type object. Calling `bool(value)` converts `value` to a boolean based on its truthiness.
`bool()` with no argument returns `false`.

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
- `str`: decimal integer parsed from the trimmed string; error if not parseable
- no argument: `0`

## float type

A 64-bit IEEE 754 double-precision floating-point number.
Float literals must contain a decimal point or an exponent (e.g. `1.0`, `1e10`).

### constructor

`float` is a type object. Calling `float(value)` constructs a float based on the type of `value`:

- `float`: same value
- `int`: same numeric value
- `bool`: `1.0` or `0.0`
- `str`: decimal float parsed from the trimmed string; error if not parseable
- no argument: `0.0`

## string type

An immutable sequence of Unicode codepoints.
String literals are enclosed in double quotes (e.g. `"hello"`).

### escape sequences

Inside a double-quoted string, the following escape sequences are recognized:

- `\n`: newline
- `\t`: tab
- `\\`: backslash
- `\"`: double quote

### constructor

`str` is a type object. Calling `str(value)` constructs a string based on the type of `value`:

- `str`: same value
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
- `str`: list of single-character strings, one per Unicode codepoint
- `dict`: list of keys
- no argument: `[]`

## dictionary type

A mutable map from string keys to values. Other key types are undefined.
Dict literals use curly braces (e.g. `{"a": 1, "b": 2}`).
Insertion order preservation is not stabilized by this spec.

### constructor

`dict` is a type object. Calling `dict(value)` constructs a dictionary based on the type of `value`:

- `dict`: shallow copy
- `list`: each element is a 2-element `[key, value]` list where `key` is a string
- no argument: `{}`

## "type" type

A type object represents a type. Type objects are first-class values and can be compared with `==`.

### constructor

`type` is a type object. Calling `type(value)` returns the type object of `value`.

Built-in type objects: `bool`, `int`, `float`, `str`, `list`, `dict`, `type`.

## operators

Operators listed from lowest to highest precedence.
`left` and `right` denote associativity of binary operators;
`prefix` and `postfix` denote unary operators.

- `=` `+=` `-=` `*=` `/=` `//=` `%=` `**=` `&=` `|=` `^=` `<<=` `>>=` (right)
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
- index/slice `[...]`, attribute `.`, call `()` (postfix)

### integer and float division

`/` always returns a float, even when both operands are integers.
`//` is floor division: the result is rounded toward negative infinity.
`%` remainder has the same sign as the divisor.

Division by zero is an error for `/`, `//`, and `%`.

### bitwise operators

`&`, `|`, `^`, `<<`, `>>` operate on integers. Both operands must be non-negative integers.

Left shift results that exceed the positive range of the integer type are an error.
Right shift with an amount at or beyond the positive range of the integer type returns `0`.

### logical operators

`and` and `or` return one of their operands, not a boolean:

- `a and b`: returns `a` if falsy, otherwise `b`
- `a or b`: returns `a` if truthy, otherwise `b`
- `not a` returns the boolean negation of `a`'s truthiness.

`and` and `or` are short-circuit.

### membership operators

`v in x` tests whether `v` is contained in `x`:

- `list`: `v` equals any element
- `str`: `v` is a substring (must be a `str`)
- `dict`: `v` is a key

`v not in x` is the negation of `v in x`.

### string and list concatenation

`a + b` concatenates two strings or two lists. For lists, the result is a new concatenated list; neither operand is modified.

### ordering

`str` values are ordered lexicographically by Unicode codepoint.
`list` values are ordered lexicographically element-by-element; a shorter list is less than a longer one if all shared elements are equal.

### equality

`list` equality is element-by-element recursive. `dict` equality is key-value recursive.

## block

A block is a sequence of expressions separated by `;`. It evaluates to its last expression.

A trailing `;` appends an implicit `null`, making the block evaluate to `null`.

## program

A program is a block.

## return

`return v` exits early, producing `v` as the result of the program.
`return` with no value produces `null`.

## control flow

### if

`if cond { ... }` evaluates the block if `cond` is truthy and returns its value.
Without an `else` branch, the result is `null` when the condition is false.
`else if` chains are supported.

### while

`while cond { ... }` loops while `cond` is truthy.

### for

`for var in expr { ... }` iterates over `expr` and binds each item to `var` for the body.

Iteration by type:

- `list`: each element in order
- `str`: each Unicode codepoint as a single-character string, in order
- `dict`: each key as a string, in insertion order


## indexing

`x[i]` accesses an element of `x`:

- `list`: `i` is an integer index. Negative indices count from the end.
- `str`: `i` is an integer index into the Unicode codepoints, returning a single-character string. Negative indices count from the end.
- `dict`: `i` is a key.

Index failure (missing key, out of bounds, wrong type) should trigger an evaluation error.

## scoping

Variables are scoped to the whole program.
A variable bound by `=` is accessible from that point to the end of the program.

## assignment

`=` is the assignment operator. It overwrites the existing binding of a name or creates a new one.

`a[i] = v` assigns to an element: an integer index into a list, or a string key into a dict.

## len

`len(x)` returns the number of elements in `x`:

- `str`: number of Unicode codepoints
- `list`: number of elements
- `dict`: number of entries

## numeric coercion

When one operand of an arithmetic or comparison operator is `int` and the other is `float`, the `int` is converted to `float` before the operation.

## mutability

All values have reference semantics: assigning a value to another variable does not copy it.
Mutations through any alias are visible through all of them.

Whether a type is effectively mutable depends solely on whether it exposes any mutating operations.

## cyclic reference

Cyclic references (e.g. a list that contains itself) are undefined behavior.

Notes: An implication is that FlowExpr may be implemented with reference counting memory management.
However, silent memory leak is usually unwanted in most embedded evaluation environments that need to be handled or at least detected.