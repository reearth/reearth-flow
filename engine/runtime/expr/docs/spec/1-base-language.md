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

### Constructor

`int` is a type object. Calling `int(value)` constructs an integer based on the type of `value`:

- `int`: same value
- `float`: truncated toward zero; error if not finite or out of range
- `bool`: `0` or `1`
- `string`: decimal integer parsed from the trimmed string; error if not parseable
- no argument: `0`

## float type

A 64-bit IEEE 754 double-precision floating-point number.
Float literals must contain a decimal point or an exponent (e.g. `1.0`, `1e10`).

## string type

An immutable sequence of Unicode codepoints.
String literals are enclosed in double quotes (e.g. `"hello"`).

## list type

A mutable, ordered sequence of values.
List literals use square brackets (e.g. `[1, "two", true]`). Elements may be of any type.

## dictionary type

A mutable, ordered map from string keys to values. Other key types are undefined.
Dict literals use curly braces (e.g. `{"a": 1, "b": 2}`).
Insertion order preservation is not stabilized by this spec.

## mutability

All values have reference semantics: assigning a value to another variable does not copy it.
Mutations through any alias are visible through all of them.

Whether a type is effectively mutable depends solely on whether it exposes any mutating operations.

## cyclic reference

Cyclic references (e.g. a list that contains itself) are undefined behavior.

Notes: An implication is that FlowExpr may be implemented with reference counting memory management.
However, silent memory leak is usually unwanted in most embeded evaluation environments that need to be handled or at least detected.