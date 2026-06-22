# Base Language

**FlowExpr** is an expression language: every program is an expression that evaluates to a single value.

This document specifies the built-in immutable types.

## null

`null` represents the absence of a value. There is exactly one null value, written `null`.

## bool

A boolean value, written `true` or `false`.

## int

A 64-bit signed integer. Integer literals match one of:

- decimal: `[0-9]+`
- binary: `0[bB][01]+`
- octal: `0[oO][0-7]+`
- hexadecimal: `0[xX][0-9a-fA-F]+`

## float

A 64-bit IEEE 754 double-precision floating-point number. Float literals must contain a decimal point or an exponent (e.g. `1.0`, `1e10`).

## string

An immutable sequence of Unicode codepoints. String literals are enclosed in double quotes (e.g. `"hello"`).
