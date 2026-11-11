# CityGML XML Attribute Representation

## Problem

The CityGML readers represent XML attributes in two different forms, controlled by two parameters:

- `keepAttributes` keeps them as `@name` keys, with the element text under `$`.
- `flattenLeafAttributes` lifts some of them to `{element}_{attribute}` keys, but only on numeric leaves with a single attribute.

Two forms mean every consumer (writer, quality checks, flatteners) has to understand both, which causes inconsistencies and extra maintenance.

## Proposal

Keep one form and one parameter: `hoistAttributes`, a list of XML attribute names.

- A listed attribute becomes a sibling key `{element}@{attribute}`, at any depth. `@` cannot appear in an XML name, so a hoisted key never collides with an element key.
- Attribute names are written as in the document, like element names: `uom`, `gml:id`.
- Attributes not listed are dropped.
- Repeated elements give an array for the element and an array for each hoisted key, matched by index, with `null` where an element has no such attribute.
- Text mixed with child elements stays under `$`.

```xml
<bldg:Building gml:id="b1">
  <my:tag lang="en" note="x">a</my:tag>
  <my:height uom="m">7.0</my:height>
  <my:tag>b</my:tag>
  <my:detail gml:id="d1">
    <my:floors>2</my:floors>
  </my:detail>
  <my:memo>A<my:ref>r1</my:ref>B</my:memo>
</bldg:Building>
```

With `hoistAttributes: [uom, lang, gml:id]`:

```json
{
  "my:tag": ["a", "b"],
  "my:tag@lang": ["en", null],
  "my:height": "7.0",
  "my:height@uom": "m",
  "my:detail": { "my:floors": "2" },
  "my:detail@gml:id": "d1",
  "my:memo": { "$": "AB", "my:ref": "r1" }
}
```

`note` is not listed, so it is dropped.

## Alternatives

The outputs below are for the example above.

### 1. Nested objects, hoisted leaves

Objects keep their attributes nested. Leaf values hoist theirs.

```json
{
  "my:tag": ["a", "b"],
  "my:tag@lang": ["en", null],
  "my:height": "7.0",
  "my:height@uom": "m",
  "my:detail": { "@gml:id": "d1", "my:floors": "2" },
  "my:memo": { "$": "AB", "my:ref": "r1" }
}
```

- Pro: closest to the current main, and the most natural.
- Con: still needs two sets of logic.

### 2. Nested objects, nested leaves with attributes

Objects stay nested. A leaf value that has any attribute matching the config also becomes an object.

```json
{
  "my:tag": [{ "@lang": "en", "$": "a" }, "b"],
  "my:height": { "@uom": "m", "$": "7.0" },
  "my:detail": { "@gml:id": "d1", "my:floors": "2" },
  "my:memo": { "$": "AB", "my:ref": "r1" }
}
```

- Pro: the simplest design, with no zipping of arrays.
- Con: every value access has to handle both the object and the flat case at runtime.

### 3. JsonML-style list

Each element is a list of name, attributes and children.

```json
["bldg:Building", { "gml:id": "b1" },
  ["my:tag", { "lang": "en", "note": "x" }, "a"],
  ["my:height", { "uom": "m" }, "7.0"],
  ["my:tag", "b"],
  ["my:detail", { "gml:id": "d1" }, ["my:floors", "2"]],
  ["my:memo", "A", ["my:ref", "r1"], "B"]
]
```

- Pro: the most faithful representation. It also solves the other lossy parts of the current object and array style.
- Con: all CityGML actions need to change, and Flow and expr need utilities to access values by key.