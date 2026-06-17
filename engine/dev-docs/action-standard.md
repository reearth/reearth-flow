# Action Standard

Reference for authoring and reviewing Re:Earth Flow actions. Covers naming, descriptions, parameters, ports, categories, and tags.

---

## How to use this standard

This standard applies to both **authoring new actions** and **reviewing existing ones**. Use §7 as a self-check before submitting a new action, and as a review checklist during audits.

**The schema is generated — never edit it directly.** All action properties (name, description, parameters, ports, categories, tags) are defined in the Rust implementation. After any change, regenerate the schema:

```bash
cargo make schema-base        # regenerates actions.json and syncs i18n skeletons
cargo make schema-translated  # regenerates per-language JSON files
```

See [engine/AGENTS.md](../AGENTS.md) for the full development workflow.

---

## 1. Names

### 1.1 Display name (`name` field)

Action names use **space-separated title case**: `Area Calculator`, `Horizontal Reprojector`, `Feature Filter`.

Rules:
- Two to four words maximum
- Plain English — avoid jargon unfamiliar to non-GIS users
- Follow the type suffix conventions in §1.2

### 1.2 Type suffix conventions

| Suffix | Use for |
|---|---|
| `... Reader` | Sources that read from a file or database |
| `... Writer` | Sinks that write to a file or database |
| `... Filter` | Actions that route features based on a condition |
| `... Extractor` | Actions that pull a value out of a feature or geometry |
| `... Calculator` | Actions that compute and attach a numeric result |
| `... Replacer` | Actions that swap geometry or an attribute value |
| `... Validator` | Actions that test features against a rule |
| `... Converter` | Actions that change the type or format of an attribute |

When none of these fit, use the most descriptive phrase available.

---

## 2. Descriptions

Verb-first, present tense — start directly with the verb, no subject.

- 1–2 sentences — prefer one; use two only when a single sentence would be genuinely unclear
- Describes what the action does to data, not how it is implemented
- Does not mention port names or internal implementation details

| ✗ | ✓ |
|---|---|
| "This processor calculates area" | "Calculates the planar or sloped area of polygon geometries" |
| "Uses the GEOS library to buffer geometries" | "Expands or contracts a geometry by a fixed distance" |
| "Routes to the `failed` port on error" | "Validates geometry against selected rules" |

---

## 3. Parameters

### 3.1 Naming

- camelCase: `outputAttribute`, `targetEpsgCode`, `groupBy`
- No abbreviations except universally understood ones: `epsg`, `crs`, `url`, `id` are fine; `attr`, `cfg`, `val` are not
- No redundant type prefixes: `stringValue` → `value`

### 3.2 Required vs optional

- A parameter is **required** if the action cannot produce meaningful output without it — it must appear in the schema's `required` array
- A parameter is **optional** if the action can run sensibly without it — whether via a schema `default` or an implementation fallback, the action must never fail at runtime when an optional parameter is absent

### 3.3 Descriptions

The parameter schema object itself must have a top-level `description` summarising what the parameter block configures. Every individual parameter property must also have a `description`.

- Prefer one sentence; two sentences are acceptable when the parameter behaviour is complex enough to warrant it
- Describes what the parameter controls and what values are valid
- Does not restate the parameter name: `"The outputAttribute"` adds nothing
- For enums, describes what each variant does — either in the property description or per-variant

### 3.4 Enum values

- camelCase: `planeArea`, `slopedArea`, `useAttributesFromOneFeature`
- No `SCREAMING_SNAKE_CASE`
- Values must be self-describing: `overwrite` not `1`, `skipExisting` not `0`

### 3.5 Parameter usability

**Minimize surface area.** Only expose parameters the user needs to control. Parameters that tune internal behavior, work around implementation constraints, or rarely deviate from a sensible default should be omitted, computed from other parameters, or fixed in code.

**Volume guideline.** More than 8 parameters is a signal to review whether any can be combined, given sensible defaults, or split into a separate action. It is not a hard cap, but it requires justification.

**Ordering.** In the schema's `properties` object, define required parameters first, followed by commonly adjusted optional parameters, followed by edge-case optional parameters last. This ordering is the foundation for future UI grouping (such as a collapsible advanced section) and makes the action easier to understand even before any grouping is added.

For example: a reprojection action puts `targetEpsgCode` (required) before `horizontalDatumTransformation` (common optional) before `axisOrder` (edge-case optional).

**No implementation leakage.** Infrastructure knobs like `timeout`, `retryCount`, `bufferSize`, or `connectionPoolSize` are internal concerns, not user controls. Omit them unless tuning them is necessary to make a workflow correct. The same applies to algorithm tuning parameters (`coordinateEpsilon`, `snapTolerance`, `maxIterations`) — expose them only when the user must adjust them for accuracy or correctness, not as a convenience for power users.

---

## 4. Ports

Port names are user-facing and appear as labels on workflow nodes.

### 4.1 Naming style

- Single-word ports: plain lowercase — `features`, `failed`, `success`, `ray`, `geom`
- Multi-word ports: kebab-case — `unjoined-requestor`, `no-intersection`, `texture-coordinates`
- No camelCase, no snake_case, no PascalCase for port names

### 4.2 Standard port vocabulary

Use these names when the semantics match. Only use custom names when the action has genuinely distinct semantics.

| Port | When to use |
|---|---|
| `features` | Primary input or output when the action has a single data stream; also the main output on actions that additionally have condition ports |
| `rejected` | Features that could not be processed (parse error, missing geometry, unexpected type) |
| `failed` | Features that were processed but did not meet a condition (validation failure, test returned false) |
| `success` | Features that satisfy a rule or validation check |
| `unfiltered` | Valid features that did not match a filter — not errors, just non-matches |
| `passed` | Features that satisfy a spatial condition |

**Multiple input ports:** When an action takes more than one input stream, both ports must have semantic names that describe their role (e.g. `requestor` + `supplier`, `base` + `overlay`). `features` is only appropriate when there is a single input.

### 4.3 Port completeness

Every feature received must be accounted for — either emitted to a named output port, or intentionally consumed to produce an output (as in merge and join operations). No feature may be silently discarded. Actions with no output ports (sinks/writers) are exempt; consuming a feature is their purpose.

- If an action can fail to process a feature (parse error, missing attribute, invalid geometry) it must have a `rejected` output port for those features
- Validators and conditional routers must route every feature to a named port
- Actions with multiple semantically distinct outputs should use descriptive names rather than `features`

---

## 5. Categories

Single category per action. Categories are the primary browsing dimension in the UI palette.

| Category | Covers |
|---|---|
| `Input` | Sources — readers of files and databases |
| `Output` | Sinks — writers to files and formats |
| `Geometry` | Geometry transformation, analysis, and validation |
| `Attribute` | Attribute creation, modification, and mapping |
| `Feature` | Feature-level operations and CityGML reading |
| `Filter` | Conditional routing, spatial and type filtering |
| `Merge` | Joining, merging, and sorting features |
| `Transform` | Expression-based and script-based transformation |
| `File` | File utilities: decompression, path handling |
| `Debug` | Development aids: echo, noop, counter |

New categories can be added when a meaningful group of actions does not fit any existing category. Avoid adding a new category for a single action.

---

## 6. Tags

- All lowercase, hyphenated if multi-word: `coordinate-system`, `citygml`
- Aim for 2–4 tags; 1 is acceptable when no second tag adds genuine discovery value. Never pad to meet a count.
- Draw from the established vocabulary below; propose additions conservatively

**Established vocabulary:**
`3d`, `aggregation`, `attribute`, `citygml`, `compression`, `coordinate-system`, `csv`, `database`, `debug`, `file`, `filter`, `geometry`, `geojson`, `geopackage`, `json`, `list`, `raster`, `routing`, `shapefile`, `spatial`, `tiling`, `validation`, `vector`, `xml`

New tags can be proposed when an established term does not adequately describe an action's domain. Avoid adding tags that duplicate an action's category.

---

## 7. Review Checklist

For each action, flag anything that violates the rules above. Only log issues — skip clean items.

```
ActionName
  name:    [proposed space-case name if different]
  desc:    [issue if any]
  params:  [list issues by param name; flag if count exceeds 8 without justification (§3.5)]
  ports:   [list issues by port name]
  cat:     [issue if wrong category]
  tags:    [missing tags | irrelevant tags]
```

If an action is clean on all dimensions, write: `ActionName — OK`
