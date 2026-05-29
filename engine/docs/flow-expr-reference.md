# FlowExpr Language Reference

> **Engine version:** 0.0.374
> **Last updated:** 2026-05-29

FlowExpr is a Python-like expression language used throughout the Re:Earth Flow engine to drive dynamic behavior in workflow actions. It supports a rich set of types, operators, control flow, and built-in functions.

---

## Table of Contents

1. [Data Types](#data-types)
2. [Literals](#literals)
3. [Variables & Assignment](#variables--assignment)
4. [Operators](#operators)
5. [Indexing & Slicing](#indexing--slicing)
6. [Method Calls](#method-calls)
7. [Control Flow](#control-flow)
8. [Blocks & Sequences](#blocks--sequences)
9. [Built-in Functions](#built-in-functions)
10. [The `Url` Type](#the-url-type)
11. [Math Functions](#math-functions)
12. [Engine Integration](#engine-integration)
13. [Design Constraints](#design-constraints)

---

## Data Types

| Type    | Rust backing | Notes |
|---------|-------------|-------|
| `null`  | `()`        |       |
| Boolean | `bool`      | `true`, `false` |
| Integer | `i64`       |       |
| Float   | `f64`       |       |
| String  | `String`    | UTF-8 |
| Array   | `Rc<Vec<Value>>` | Reference-counted |
| Map     | `Rc<IndexMap<String,Value>>` | Ordered, reference-counted |
| Url     | Custom      | Path / URI manipulation |
| Function | Native Rust | Callable from expressions |

---

## Literals

```python
42              # integer
3.14            # float
"hello\nworld"  # string — escape sequences: \n \t \\ \" \'
true            # boolean
false
null
[1, 2, 3]       # array literal
{x: 10, y: 20}  # map literal
```

---

## Variables & Assignment

```python
x = 10                # assign and return the value
arr[0] = 5            # index assignment
map["key"] = value    # map key assignment

# Compound assignment operators
x += 1    # x = x + 1
x -= 1    # x = x - 1
x *= 2    # x = x * 2
x /= 2    # x = x / 2
x //= 2   # x = x // 2  (floor division)
x %= 3    # x = x % 3
x **= 2   # x = x ** 2
```

---

## Operators

### Arithmetic

| Operator | Meaning |
|----------|---------|
| `+` | Addition / string concat |
| `-` | Subtraction |
| `*` | Multiplication |
| `/` | Division |
| `//` | Floor division |
| `%` | Modulo |
| `**` | Exponentiation (right-associative) |
| `-x` | Unary negation |

### Comparison

| Operator | Meaning |
|----------|---------|
| `==` | Equal |
| `!=` | Not equal |
| `<` | Less than |
| `<=` | Less than or equal |
| `>` | Greater than |
| `>=` | Greater than or equal |

### Logical

| Operator | Meaning |
|----------|---------|
| `and` | Logical AND |
| `or` | Logical OR |
| `not` | Logical NOT |

### Membership

| Operator | Meaning |
|----------|---------|
| `in` | `"a" in ["a","b"]` |
| `not in` | Negated membership |

### Precedence (lowest → highest)

1. Assignment (`=`, `+=`, …)
2. `or`
3. `and`
4. `not`
5. `==` `!=` `in` `not in`
6. `<` `<=` `>` `>=`
7. `+` `-`
8. `*` `/` `//` `%`
9. Unary `-` / `not`
10. `**` (right-associative)
11. Postfix (index, slice, method call)

---

## Indexing & Slicing

```python
arr[0]          # zero-based index
map["key"]      # map key lookup
feature["attr"] # feature attribute (map syntax)

# Python-style slices
"hello"[1:3]    # "el"
arr[1:5:2]      # start:stop:step
arr[::2]        # every second element
```

---

## Method Calls

### String methods

```python
"hello".len()
"hello".split(",")
"  hello  ".trim()
"hello".starts_with("he")
"hello".ends_with("lo")
"hello".replace("l", "L")
"prefix_value".remove_prefix("prefix_")
"value_suffix".remove_suffix("_suffix")
```

### Array methods

```python
[1,2,3].len()
```

### Map methods

```python
my_map.len()
my_map.keys()     # returns array of keys
my_map.values()   # returns array of values
my_map.items()    # returns array of [key, value] pairs
```

### Url methods (see [The `Url` Type](#the-url-type))

```python
Url("/path/to/file.txt").name()       # "file.txt"
Url("/path/to/file.txt").stem()       # "file" (no extension)
Url("/path/to/file.txt").extension()  # "txt"
Url("/path/to/file").parent()         # "/path/to"
```

---

## Control Flow

### if / else if / else

```python
if condition {
  then_expr
} else {
  else_expr
}

if x > 10 { "high" }
else if x > 5 { "medium" }
else { "low" }
```

### while loop

```python
while i < 10 {
  i = i + 1
}
```

### for-in loop

```python
for item in list {
  print(item)
}
```

---

## Blocks & Sequences

A block `{ … }` evaluates to its final expression. Statements are separated by `;`.

```python
{
  x = 10;
  y = 20;
  x + y       # block evaluates to 30
}

{
  x = 10;     # trailing semicolon → block evaluates to null
}
```

---

## Built-in Functions

### Type conversion

| Function | Description |
|----------|-------------|
| `str(v)` | Convert to string |
| `int(v)` | Convert to integer |
| `float(v)` | Convert to float |
| `bool(v)` | Convert to boolean |
| `list(v)` | Convert to array |
| `map(v)` | Convert to map |

### Engine integration

| Function | Description |
|----------|-------------|
| `value("attr")` | Read a feature attribute by name |
| `env("VAR")` | Read an environment variable |
| `Url(path)` | Construct a [Url](#the-url-type) value |
| `print(...)` | Debug print; returns first argument |

---

## The `Url` Type

`Url` represents a file system or cloud storage path and is used extensively in actions that deal with file I/O.

```python
Url("/path/to/file.txt")          # construct from string
Url(env("BASE_DIR")) / "subdir"   # path concatenation with /
Url("gs://bucket/key") / "file"   # works with GCS URIs
str(Url("/path"))                 # convert back to string

# Methods
Url("/path/to/file.txt").name()       # "file.txt"
Url("/path/to/file.txt").stem()       # "file"
Url("/path/to/file.txt").extension()  # "txt"
Url("/path/to/file.txt").parent()     # "/path/to"
```

---

## Math Functions

All math functions live in the `math::` namespace.

### Constants

| Name | Value |
|------|-------|
| `math::PI` | π ≈ 3.14159… |
| `math::E` | e ≈ 2.71828… |
| `math::TAU` | 2π ≈ 6.28318… |

### Functions

| Category | Functions |
|----------|-----------|
| Trigonometry | `sin` `cos` `tan` `asin` `acos` `atan` `atan2` |
| Hyperbolic | `sinh` `cosh` `tanh` `asinh` `acosh` `atanh` |
| Exponential / Log | `exp` `exp_m1` `ln` `ln_1p` `log` `log10` `log2` |
| Power / Root | `sqrt` `cbrt` `pow` `hypot` |
| Rounding | `floor` `ceil` `round` |
| Comparison | `abs` `min` `max` |
| Angle conversion | `to_radians` `to_degrees` |
| Other | `copysign` |

All functions accept `float` arguments. Example:

```python
math::sqrt(math::pow(a, 2.0) + math::pow(b, 2.0))  # hypotenuse
math::to_radians(45.0)
math::atan2(y, x)
```

See [`expression-math-functions.md`](expression-math-functions.md) for full signatures.

---

## Engine Integration

### Compile & evaluate

```rust
let compiled = compile(input_str)?;
let value    = eval(&compiled, &mut env)?;
let s        = eval_string(&compiled, &mut env)?;
```

### Code type

Actions that accept expressions use the `Code` enum:

```rust
Code::FlowExpr(expr_string)  // evaluated by this language
Code::String(literal)        // treated as a plain string value
```

### Used in

- `AttributeMapper` — map/transform feature attributes
- `FeatureFilter` — boolean filter predicate
- `FlowExprTest` — unit-testable expression nodes
- `Cesium3DTilesWriter`, `MVTWriter`, and other sink actions

---

## Design Constraints

| Constraint | Detail |
|------------|--------|
| **No iteration limit on `while`** | Relies on action-level timeouts |
| **No cycle detection** | Circular references in maps/arrays are undefined behavior |
| **Memory model** | Arrays and maps use `Rc` (reference counting); no tracing GC |
| **Recursion depth** | 64 in debug builds, 1024 in release builds |
| **No `import` / modules** | All built-ins are pre-registered in the environment |
