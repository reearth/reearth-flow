# Design: Replacing RJSF with a purpose-built schema form

Status: **Done** — RJSF removed; `src/lib/schemaForm/` + `src/components/SchemaForm/`
Owner: UI
Scope: `ui/src/components/SchemaForm/`, `ui/src/features/Editor/components/ParamsDialog/`

## Summary

Replace `@rjsf/core` with a small, in-house renderer that **compiles** an action
parameter schema into an explicit field tree, then renders that tree. Keep AJV,
but validate against the **original** schema instead of a patched one.

The point is not that RJSF is bad. The point is that RJSF is a _general_ JSON
Schema form engine, and our input is not general JSON Schema — it is the narrow
dialect that Rust's `schemars` emits. Because RJSF resolves combinators its own
way, every mismatch has to be fixed _before_ RJSF sees the schema, which is why
`patchSchemaTypes.ts` exists and why it keeps growing. A compile step lets us
recognise those shapes directly instead of rewriting the schema until RJSF
guesses right.

## Background: what we actually feed the form

Measured against `engine/schema/actions.json` (171 actions, 141 with a
`parameter` schema):

| Keyword                                   | Sites | Notes                                                                                                |
| ----------------------------------------- | ----- | ---------------------------------------------------------------------------------------------------- |
| `allOf`                                   | 105   | **every single one** is `allOf: [{$ref}]` — the schemars "ref + title/default" wrapper               |
| `oneOf`                                   | 64    | 42 all-branches-are-`enum`-of-one (tagged string enum), 18 all-object (discriminated union), 4 mixed |
| `anyOf`                                   | 38    | 37 are `[{$ref}, {"type":"null"}]` (nullable ref); 1 is `[boolean, number, string]`                  |
| `definitions`                             | 89    | `Attribute` alone accounts for 57                                                                    |
| `format`                                  | 215   | `code` ×149 (FlowExpr/Code), the rest numeric width hints (`double`, `uint64`, …)                    |
| `type: [X, "null"]`                       | 172   | nullable scalars/objects/arrays                                                                      |
| `additionalProperties`                    | 8     | 7 are `false` (closed union branches), 1 is a real map type                                          |
| `minimum`/`maximum`/`minItems`/`maxItems` | 41    |                                                                                                      |

Not present anywhere in the corpus: `if`/`then`/`else`, `dependencies`,
`patternProperties`, `not`, `contains`, tuple `items`, external `$ref`,
`$defs`, `unevaluated*`, `const` outside a `oneOf` branch.

Nesting depth (with `$ref` resolved): 40 schemas at depth 1, 85 at depth 2,
7 at depth 3, 8 at depth 4, 1 at depth 6.

**This is a closed, small, machine-generated grammar.** Roughly eight shapes
cover 100% of it. That is what makes a purpose-built renderer tractable — and
it is exactly the information RJSF cannot use.

## Problem

### 1. Nullable is not representable — measured, and it is the headline bug

`simplifyAnyOf` collapses `anyOf: [{$ref: X}, {"type": "null"}]` to `{$ref: X}`.
The null branch is **deleted**, so "unset" stops being a legal value. Minimal
repro, run against the real patch code:

```
ORIGINAL  encoding: { anyOf: [ {$ref: Enc}, {type: "null"} ] }
PATCHED   encoding: { $ref: Enc, title: "Encoding" }
PATCHED   Enc:      { type: "string", oneOf: [{const:"text"},{const:"base64"}] }

  {"encoding": null}     original=VALID    patched=INVALID  ← cannot express "unset"
  {"encoding": "text"}   original=VALID    patched=VALID
  {}                     original=VALID    patched=VALID

  getDefaultFormState({}) on original = {}
  getDefaultFormState({}) on patched  = {"encoding": "text"}   ← invented a value
```

Three consequences, in increasing severity:

**a. Valid configs are reported invalid.** `HTTP Caller` with
`response.responseEncoding: null` validates clean against the engine's schema
and produces three AJV errors against the patched one (`must be string`,
`must be equal to constant`, `must match exactly one schema in oneOf`).

**b. The "empty" option does not produce null.** `SelectWidget` does render a
`-` option (because the field is not `required`), but picking it calls
`onChange(options.emptyValue)`, which is `undefined` — measured. So the round
trip is `null → "-" → undefined`, three different representations of one state.

**c. A value is fabricated where the schema had none.** With the null branch
gone, RJSF's `getDefaultFormState` sees `type: "string"` + `oneOf`, picks option
0, and materializes its `const`. For `responseEncoding` that means the form
silently writes `"text"` — and the field's own description says _"When omitted,
the encoding is chosen from the response's Content-Type header."_ Opening the
params dialog therefore turns off content-type sniffing and pins responses to
UTF-8 text, mangling binary payloads. **Nobody touched the field.**

The same collapse applies to nullable _objects_ and _unions_, which is the
larger blast radius. Census across the 141 parameter schemas:

| Nullable shape (`anyOf: [X, null]`)   | Sites | Example                                                                                                                              |
| ------------------------------------- | ----- | ------------------------------------------------------------------------------------------------------------------------------------ |
| → scalar (`Attribute`, `MappedValue`) | 21    | most actions                                                                                                                         |
| → union of objects                    | 8     | `HTTP Caller.authentication`, `CSV Reader.geometry`                                                                                  |
| → plain object                        | 5     | `HTTP Caller.retry`, `.rateLimit`, `.timeouts`                                                                                       |
| → enum                                | 3     | `HTTP Caller#ResponseConfig.responseEncoding`, `Feature Joiner.conflictResolution`, `Attribute Table Extractor#ExtractRule.dataType` |

For `HTTP Caller`, every optional section becomes mandatory. Mounting the form
with **no saved params at all** writes this back through `onChange` — into Yjs:

```json
{
  "url": {},
  "method": "GET",
  "authentication": { "username": {}, "password": {} },
  "requestBody": { "content": {} },
  "response": {
    "responseBodyAttribute": "_response_body",
    "responseEncoding": "text"
  },
  "retry": {
    "maxAttempts": 3,
    "initialDelayMs": 100,
    "backoffMultiplier": 2,
    "maxDelayMs": 10000,
    "honorRetryAfter": true
  },
  "rateLimit": { "intervalMs": 1000, "timing": "burst" }
}
```

Against the original schema the correct answer is `{"url": {}, "method": "GET"}`.
The injected `rateLimit` is missing its own required `requests`, so the form
writes data that is invalid by its _own_ patched schema. Measured AJV output for
a minimal valid config (`url` only) is 11 errors, none of which correspond to
anything the user did.

**d. And the validation is inert anyway.** `onValidationChange` fires
`true → false` on mount, then flips back to `true` on the first keystroke in any
field — because RJSF's `onChange` carries no errors without `liveValidate`, so
`handleChange` clears the flag. `isCurrentTabValid` gates the Update button
(`ParamEditor` line ~230, and `handleUpdate` early-returns on it). Net effect:
the dialog shows "Invalid data" and a disabled Update on open, both of which
vanish when you type a character somewhere unrelated. It blocks correct configs
and fails to block incorrect ones.

This is not a patch that needs a fifth pass. Nullability is _information the
target representation cannot hold_, so no amount of rewriting recovers it.

### 2. The patch pipeline is load-bearing and still wrong

`patchSchemaTypes.ts` is four passes applied in a fixed order:

```
simplifyAnyOf            → drop null branches, collapse 1-branch anyOf
simplifyAnyOfInsideOneOf → re-run simplifyAnyOf on root-level oneOf branches
simplifyAllOf            → resolve allOf:[{$ref}] against definitions
consolidateOneOfToEnum   → turn const/enum-only oneOf into enum or const+title
```

Each pass was added by a separate bug-fix PR (#722, #949, #1364, #1499, #1518,
#2187) over roughly two years. They do not share a traversal, and their
traversals disagree:

- `simplifyAnyOf`, `simplifyAllOf` and `consolidateOneOfToEnum` recurse into
  `properties`, `items` and `definitions` — but **never into `oneOf` or `anyOf`
  branches**.
- `simplifyAnyOfInsideOneOf` was added to cover that, but it only inspects
  `schema.oneOf` **at the root**, and only re-applies `simplifyAnyOf` — not
  `simplifyAllOf` or `consolidateOneOfToEnum`.

Net effect: inside a `oneOf` object branch, nothing is normalised. Confirmed
against the corpus — 13 sites across 5 actions have an unresolved
`allOf: [{$ref}]` sitting inside a `oneOf` branch today:

```
Coordinate Extractor           #CoordinateExtractionMode.oneOf[0].coordinatesListName
HTTP Caller                    #Authentication.oneOf[2].location
JSON Fragmenter                .oneOf[0].jsonAttribute
PLATEAU4.SolarPositionCalculator  .oneOf[0].outputType
XML Fragmenter                 .oneOf[0].attribute
```

Adding a fifth pass fixes this class of bug. It does not fix the next one. The
passes are a fixed-point computation nobody is computing a fixed point for.

### 3. Two schemas exist, and the code knows it

`SchemaForm` threads **both** `originalSchema` and `patchedSchema` through
`formContext`, because patching destroys information the UI needs:

- UI-schema generation (`buildExprUiSchema`) must run on the _original_ schema,
  since `anyOf: [{$ref: Expr}, {null}]` is how an Expr field is recognised — and
  `simplifyAnyOf` deletes exactly that.
- `BaseInputTemplate` then has a fallback that scans **every definition in the
  schema** for _any_ property with the same `name`, to decide whether the field
  it is currently rendering is an Expr:

  ```ts
  hasExprSupport = Object.values(originalSchema.definitions).some(
    (def) => def?.properties?.[name]?.$ref === "#/definitions/Expr" || ...
  );
  ```

  That is a name-collision waiting to happen (369 distinct property names across
  the corpus). It exists only because the patched node no longer knows where it
  came from.

### 4. Validation validates the wrong thing

`validator.validateFormData(formData, patchedSchema)` — the form's notion of
"valid" is AJV over a schema the engine has never seen. Nullability has been
stripped, `oneOf` has been rewritten to `enum`, `allOf` has been merged. The
form can report valid for data the engine rejects, and the "Update" button is
gated on that result.

### 5. Union handling is a guess

The 22 object-`oneOf` sites are discriminated unions. schemars gives us the
discriminator explicitly:

```json
{ "type": "object", "required": ["type"],
  "properties": { "type": { "type": "string", "enum": ["allCoordinates"] }, ... } }
```

RJSF instead picks a branch with `getMatchingOption`, which scores branches by
validating the current form data against each. Consequences: variant labels come
out inconsistent, and switching variants discards field values that the target
variant also has (`xColumn` survives a WKT↔Coordinates toggle in the schema, not
in the form).

### 6. Coupling to RJSF internals

Field identity across the whole collaborative-awareness layer is RJSF's DOM id
string, reconstructed by splitting on `_`:

```ts
export function extractFieldPath(id: string): string[] {
  return id
    .replace(/^root_/, "")
    .split("_")
    .filter(Boolean);
}
```

This works today only because serde's `rename_all = "camelCase"` means no
property name in the corpus contains `_`. One snake_case field in one action
silently corrupts the awareness map and the value-editor write path. The v6
upgrade (`fieldPathId`, template renames) was also absorbed across ~15 template
files — that recurs on every major.

## Non-goals

- Supporting arbitrary JSON Schema. We support the schemars dialect plus an
  explicit escape hatch.
- Changing the engine's schema output. (Discussed under Alternatives.)
- Changing the Yjs/awareness protocol, the params dialog UX, or the FlowExpr
  editor.
- Rewriting the widgets. They are already ours and mostly port over.

## Proposal

Three stages, each independently testable, replacing "patch the schema until
RJSF agrees".

```
JSON Schema ──normalize──▶ NormalizedNode ──compile──▶ FieldNode ──render──▶ React
     │                                                               │
     └──────────────── AJV validates the ORIGINAL ───────────────────┘
```

### Stage 1 — `normalize`

One recursive walker, applied uniformly at **every** position (properties,
items, `additionalProperties`, and inside every `oneOf`/`anyOf`/`allOf` branch —
this is the bug class from Problem 2, removed by construction):

- resolve `$ref` against `definitions` (with a cycle guard)
- merge `allOf: [{$ref: X}]` — referent first, local keywords win
- lift nullability out of `type: [X, "null"]` and `anyOf: [X, {type:"null"}]`
  into an explicit `nullable: boolean`, **without discarding the branch**
- leave `oneOf` intact — it is meaning, not noise

Output is a plain tree with `nullable`, resolved `type`, and a `path`
(`(string | number)[]`) on every node. Idempotent and pure; trivially
snapshot-testable over all 141 schemas.

### Stage 2 — `compile`

Classify each normalized node into one of a closed set of kinds:

```ts
type FieldNode =
  | { kind: "string";  path: FieldPath; nullable: boolean; ... }
  | { kind: "number";  integer: boolean; min?: number; max?: number; ... }
  | { kind: "boolean" }
  | { kind: "enum";    options: { value: unknown; label: string }[] }
  | { kind: "array";   item: FieldNode; minItems?: number; maxItems?: number }
  | { kind: "object";  properties: FieldNode[]; required: string[] }
  | { kind: "map";     value: FieldNode }            // additionalProperties: <schema>
  | { kind: "union";   variants: Variant[]; match: (v: unknown) => number }
  | { kind: "expr";    flavor: "flowExpr" | "python" | "code" }
  | { kind: "color" } | { kind: "wysiwyg" } | { kind: "asset" }
  | { kind: "unsupported"; reason: string; schema: JSONSchema7 };
```

Two things to call out:

**`union` is first-class.** Each variant carries its discriminator when there is
one:

```ts
type Variant = {
  key: string;
  title: string;
  description?: string;
  discriminator?: { prop: string; value: unknown }; // tagged
  requiredKeys: string[]; // untagged fallback
  node: FieldNode;
};
```

Variant selection is deterministic, in priority order: exact discriminator match
→ required-key match (`additionalProperties: false` narrows this) → first
variant. No validation-scoring. Variant _switching_ preserves keys shared with
the target variant and stashes the outgoing variant's data in component state,
so toggling back is non-destructive.

**`expr` is decided here, once, from the original shape** — `format: "code"`,
`$ref: #/definitions/Expr`, or either wrapped in `allOf`/`anyOf`. That deletes
`buildExprUiSchema`, the dual-schema `formContext`, and the
scan-all-definitions-by-name fallback in `BaseInputTemplate`. Python-vs-FlowExpr
stays an explicit `(actionName, path)` override, but in a declared table rather
than an `if` buried in a template.

**`unsupported` is a feature.** Anything the compiler does not recognise becomes
a labelled raw-JSON editor with the validation errors attached. No blank field,
no crash, no silent data loss — and it is the safety net for user-authored
custom action schemas.

### Stage 3 — `render`

A `switch` on `kind`. No `registry`, no `getTemplate`, no `withTheme`, no
`uiSchema` protocol. Props are explicit:

```tsx
<Field node={node} value={valueAt(node.path)} errors={errorsFor(node.path)}
       focusedUsers={focusMap[key(node.path)]} readonly={readonly}
       onChange={(v) => setAt(node.path, v)} onFocus={...} />
```

Existing widgets (`SelectWidget`, `CheckboxWidget`, `TextInput`, `NumberInput`,
`ColorInput`, `WysiwygWidget`, `FlowExprWidget`, …) port over by swapping the
`WidgetProps` shape for the above. They already do their own awareness styling
and their own DropdownMenu rendering — very little of that code is RJSF-shaped.

### Validation

AJV against the **original** schema. `instancePath` maps directly to
`FieldPath`, so errors attach to nodes without a translation table. The form's
verdict and the engine's verdict become the same verdict.

Defaults move to `applyDefaults(FieldNode, value)` over the compiled tree, which
knows real nullability — replacing `schemaUtils.getDefaultFormState` over the
patched (nullability-stripped) schema.

> Confirmed, not speculation: `getDefaultFormState` over the patched schema
> injects whole optional sections and invents enum values, and `SchemaForm`
> writes the result back through `onChange` on mount. See Problem 1.

### Field identity

`FieldPath = (string | number)[]`, serialised as a JSON Pointer
(`/mode/coordinatesListName`, `/rules/0/attribute`) for the awareness map key
and for `FieldContext`. `extractFieldPath` and its underscore assumption go
away. This is a wire-format change for awareness keys — see Risks.

## What we give up

| RJSF feature                                            | Corpus usage  | Plan                                                        |
| ------------------------------------------------------- | ------------- | ----------------------------------------------------------- |
| `additionalProperties` key add/rename UI                | 1 real site   | Port as the `map` kind; drop `WrapIfAdditionalTemplate`     |
| Array add/remove/reorder/copy                           | 29 arrays     | Keep — our own `ArrayFieldTemplate` already implements it   |
| `ui:` schema ecosystem                                  | internal only | Replaced by compile-time classification + an override table |
| `if`/`then`/`else`, `dependencies`, `patternProperties` | 0             | Not implemented; `unsupported` catches them                 |
| Upstream bug fixes                                      | —             | We own the bugs now. That is the trade.                     |

Dependency delta: drop `@rjsf/core`, `@rjsf/utils`, `@rjsf/validator-ajv8`; keep
`ajv` + `ajv-formats` directly.

## Alternatives considered

**Fix forward on RJSF.** The tracking issue for the nullable-`anyOf` behaviour
([rjsf#4380](https://github.com/rjsf-team/react-jsonschema-form/issues/4380)) is
cited in our own source and is still open. Even with it fixed, the dual-schema
problem, the union-matching heuristic, and the id-string coupling remain — those
are design choices, not bugs.

**Emit a Flow-specific form model from the engine.** Genuinely attractive: the
engine already owns `actions.json`, and a `x-flow-ui` annotation (or a
purpose-built form schema) would remove all guessing at the source. Rejected
_for now_ because it couples every UI form change to an engine release, and it
does nothing for user-authored custom action schemas. Note that the compile
stage makes it a cheap follow-up: the compiler can honour `x-flow-*` hints when
present and fall back to inference when absent.

**Swap to another library (JSONForms, react-hook-form + resolver).** JSONForms
has the same generality-vs-dialect mismatch with a different renderer protocol —
we would be re-learning a second framework's opinions about `oneOf`.
react-hook-form solves state, not schema interpretation; we would still write
stages 1 and 2.

**Do nothing.** Defensible if params UI is stable. It is not — the corpus is
growing (PLATEAU 4/6 actions landed recently, CityGML readers before that), and
discriminated unions are the pattern new actions reach for.

## Plan

**Phase 0 — baseline (before writing renderer code).**
Golden-corpus harness: walk all 171 actions, render each parameter schema under
today's RJSF, snapshot the resulting field structure. This is the regression
oracle for every later phase. Cheap, and it has standalone value.

**Phase 1 — `normalize` + `compile`, no UI. ✅ Done.**
`src/lib/schemaForm/` — `types.ts`, `path.ts`, `normalize.ts`, `compile.ts`,
plus `nullable.test.ts`, `reportedBugs.test.ts` and `corpus.test.ts`. The CI gate
holds: all 141 parameter schemas compile with **zero** `unsupported` nodes.

```
compiled 141 action schemas
    213  object      69  number      26  union       217  nullable fields
    213  string      68  boolean      1  map          17  tagged unions
    151  expr        59  enum                          9  untagged unions
                     57  array
```

Two shapes needed work that the corpus survey had not predicted, both caught by
the gate rather than by inspection:

- a sole surviving `anyOf` branch that is itself a choice (`anyOf: [{$ref:
MappedValue}, null]`, where `MappedValue` is an untagged union of scalars) —
  the fold has to carry the inner choice up rather than flatten it away;
- `additionalProperties` as a real map (`Attribute Table Extractor.inline`),
  which is the single `map` node in the table above.

`normalize` also resolves the 13 `allOf`-inside-`oneOf` sites by construction,
and `compile` drops each union's discriminator property from its rendered
fields, which removes the redundant second dropdown.

**Phase 2–4 — renderer, port, removal. ✅ Done.**
Shipped in one pass rather than behind a flag: the compiler's corpus gate made a
per-action allowlist redundant, since every schema was known to compile before
any of it rendered.

- `src/components/SchemaForm/fields/` — one component per field kind and a
  single `Field` switch. No registry, no templates, no `ui:` protocol.
- `src/components/SchemaForm/context.ts` — errors, awareness and the editor
  callbacks, passed explicitly instead of through a framework registry.
- Removed: `patchSchemaTypes.ts`, `Templates/` (24 files), `Widgets/` (9),
  `ThemedForm.tsx`, `Fields/FlowExprField.tsx`, `utils/WrapIfAdditionalTemplate.tsx`,
  `utils/fieldUtils.ts`, and the `@rjsf/core`, `@rjsf/utils`,
  `@rjsf/validator-ajv8` dependencies. `ajv` and `ajv-formats` are now direct.
- `RJSFSchema` is replaced by `FlowSchema` across `types/`, `hooks/` and the
  params dialog.

**What the field keys do on the wire.** Focus and draft patches key on the dot
path (`response.responseEncoding`). That is already the spelling `paramDrafts`
uses, so the collaborative wire format is unchanged and the risk flagged below
did not materialise — the only cross-build difference is the awareness focus id,
where a mismatch costs a highlight, not data. `parsePathKey` reads a numeric
segment back as a number, so a patch into `rules.0.name` now rebuilds an array
rather than an object keyed `"0"`.

**Five bugs the work surfaced, none of them predicted:**

1. `compile(null)` threw. 30 of the 171 actions publish `parameter: null`, and
   `buildNewCanvasNode` hands that straight through every time one of them is
   dropped on the canvas — so adding any parameterless action would have
   crashed. The old patch tolerated it by accident (`{...undefined}`).
2. Field labels were not associated with their controls. `FieldRow` now renders
   a real `<label htmlFor>`; the previous templates used a bare `<p>`.
3. A nullable section that was present but incomplete reported `must be null`
   and `choose one of the available options` on itself, on top of the child's
   real error — both artefacts of AJV walking the null branch of a nullable
   `anyOf`. `validate` drops them where something more specific is known.
4. Four actions — Feature Reader, JSON Fragmenter, XML Fragmenter,
   PLATEAU4.SolarPositionCalculator — have a union rather than an object at the
   root of their schema, so choosing a variant changes the whole params object
   at path `""`. `ParamsDialog` rejected that as falsy and dropped the edit in
   silence. `""` is now a path like any other.
5. Adding a row to an array of objects produced `undefined` rather than `{}`,
   because seeding an object with no defaults of its own from `undefined`
   returns `undefined`. The row landed in the array as a hole and read back as
   `must be object`. Found by comparing a screenshot of the new form against
   the old one, not by a test.

**Behaviour that deliberately changed:**

- **Update is now genuinely gated.** `ParamEditor` has always disabled Update on
  `!isCurrentTabValid`; it just never meant anything, because the old form
  reported `false` on mount and then `true` again on the first keystroke
  anywhere. Validation now tracks the engine's schema, so an action with a
  required field unfilled cannot be saved. If a half-configured node should
  still be savable, that is one line in `ParamEditor`.
- **Errors are held back until the user engages**, so opening a partly-filled
  action does not greet them with errors they did not cause.
- **A required field left blank is shown as a highlight, not a sentence.** The
  asterisk beside the label and the red border on the control already say it;
  writing "This field is required" underneath is the third telling. A path in
  `ValidationErrors` may therefore map to an empty message list — the key's
  presence is what makes the field invalid, the list is only what to write
  beneath it. `useField` splits the two questions: `hasErrors` drives the
  border and `aria-invalid`, `errors` drives the text. Messages that say
  something a border cannot ("Not one of the allowed values") are unaffected.
- **Discriminators are hidden.** A union renders one dropdown, not a variant
  picker plus a required `type` dropdown repeating the same choice.
- **Optional sections start collapsed**, with an Add button (objects) or a
  "Not set" option (unions and enums), and can be removed again.
- **Leaf descriptions are not repeated under every input.** Sections keep
  theirs, and the form's title tooltip keeps the summary; a leaf's description
  under its control tripled the height of a list like Feature Filter's
  conditions. This matches what the old templates rendered — `FieldTemplate`
  drew label, control, errors and help, never a description.
- `onEditorOpen` / `ValueEditorDialog` is now unreachable. It already was: it
  required a field typed `$ref: "#/definitions/Expr"`, which the engine stopped
  emitting — all 149 expression fields are `format: "code"`. The prop is kept
  plumbed and documented rather than removed.

**Tests.** 93 across the three directories: the compiler's corpus gate, the
reported bugs pinned against the real schemas, defaults, validation, path
helpers, and the form rendered and driven with `@testing-library/user-event` —
including every one of the 141 parameter schemas rendered without throwing, and
the draft-patch key format that rides over Yjs.

## Risks

| Risk                                                                                                 | Mitigation                                                                                                                                                           |
| ---------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Custom action schemas are user-authored** and need not follow the schemars dialect                 | `unsupported` → raw JSON editor; validation against the real schema still works. Keep RJSF for `customizations` until parity                                         |
| **Awareness key format change** mid-rollout — clients on different builds disagree on focus-map keys | Ship behind the same flag as the renderer; worst case is a stale focus highlight, not data loss. Confirm before Phase 3                                              |
| **Unknown RJSF behaviours we depend on without knowing**                                             | That is what Phase 0's golden corpus is for                                                                                                                          |
| **Scope creep into UX redesign**                                                                     | Explicit non-goal. Phase 2 parity means _identical_ rendering, improvements come after                                                                               |
| **We now own the bugs**                                                                              | Accepted. The bugs we own are in ~1.4k lines of code shaped like our problem, versus a general engine plus a 250-line patch layer that is already wrong in 13 places |

## Open questions

1. Do we want the engine to eventually emit UI hints (`x-flow-ui`), and should
   the compiler be built to accept them from day one? (Cheap now, expensive to
   retrofit.)
2. Should `unsupported` be a hard CI failure for built-in actions but a soft
   fallback for custom actions? (Proposed: yes.)
3. The single `anyOf: [boolean, number, string]` site — real union, or should the
   engine tighten it?
4. Do the workflow-variables editors (`ChoiceEditor`, `ArrayEditor`, …) migrate
   with the params dialog, or stay on RJSF indefinitely?

## References

- `ui/src/components/SchemaForm/patchSchemaTypes.ts` — the current patch layer
- `ui/src/components/SchemaForm/index.tsx` — `buildExprUiSchema`, dual-schema `formContext`
- `ui/src/components/SchemaForm/Templates/BaseInputTemplate.tsx` — the scan-by-name Expr fallback
- `ui/src/features/Editor/components/ParamsDialog/utils/fieldUtils.ts` — `extractFieldPath`
- `engine/schema/actions.json` — the corpus this document measures
