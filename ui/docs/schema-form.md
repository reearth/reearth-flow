# Schema Form — Architecture

The action parameter form. A schema is **compiled once** into a field tree, and
that tree is rendered. There is no general-purpose form engine underneath it —
the form was built on `@rjsf/core` until it was replaced, and nothing here is
`ui:`-schema shaped.

The input is not arbitrary JSON Schema. It is the narrow dialect Rust's
`schemars` emits into `engine/schema/actions.json`, plus whatever a user writes
in a custom action. That narrowness is what makes compiling it tractable.

## Pipeline

```
JSON Schema ──normalize──▶ NormalizedNode ──compile──▶ FieldNode ──render──▶ React
     │                                                                │
     └──────────────── AJV validates the ORIGINAL ────────────────────┘
```

Validation never sees a rewritten schema, so what the form calls valid is what
the engine calls valid.

## Files

`src/lib/schemaForm/` — no React:

| File           | Role                                                                               |
| -------------- | ---------------------------------------------------------------------------------- |
| `types.ts`     | `FieldNode` union, `FieldPath`, `FlowSchema`                                       |
| `normalize.ts` | Resolve `$ref`, merge `allOf`, lift nullability out of `type: [X, "null"]`/`anyOf` |
| `compile.ts`   | Classify into field kinds; `selectVariant` picks a union's branch                  |
| `defaults.ts`  | `applyDefaults` — apply the defaults the schema states, and nothing else           |
| `validate.ts`  | AJV against the original schema; errors keyed by dot path                          |
| `migrate.ts`   | `migrateValue` — carry stored params across a schema change                        |
| `path.ts`      | `FieldPath` ⇄ dot path, and immutable get/set/delete                               |

`src/components/SchemaForm/` — the renderer:

| File               | Role                                                               |
| ------------------ | ------------------------------------------------------------------ |
| `index.tsx`        | `SchemaForm` — compiles, seeds, validates, owns the change handler |
| `context.ts`       | Errors, awareness, editor callbacks; `EditorContext`               |
| `fields/Field.tsx` | One `switch` over `kind`. The whole dispatch mechanism             |
| `fields/*.tsx`     | One component per kind                                             |
| `fields/FieldRow`  | Label + control + errors, shared by every leaf                     |
| `fields/useField`  | Per-field key, errors, awareness style, focus handlers             |

## Field kinds

`string` `number` `boolean` `enum` `array` `object` `map` `union` `expr`
`color` `wysiwyg` `unsupported`

`unsupported` is a feature, not a failure mode: it renders a labelled raw-JSON
editor so an unrecognised shape costs the user a nicer control, never their
data. Built-in actions are held to **zero** of them by `corpus.test.ts`.

## Invariants

Break these and the failures are subtle rather than loud.

**Nullability is carried, never dropped.** `nullable` says whether "unset" is a
legal value. Collapsing `anyOf: [X, null]` to `X` makes "unset" unrepresentable,
turns every optional section into a mandatory one, and lets defaults invent
values the schema never had.

**Validation runs against the schema as published.** Never validate a
normalized, compiled or otherwise rewritten copy.

**`null` is a value, not an absence.** `applyDefaults` and `migrateValue` both
leave it alone. Seeding into it turns an explicit "unset" into `{}`.

**Clearing a field deletes its key.** Writing `undefined` leaves the key
present — `Object.keys` still reports it, so an object with
`additionalProperties: false` rejects it, and it rides into the saved params as
a phantom entry. `SchemaForm.handleChange` and `applyMergedPatch` both use
`deleteAtPath` for this.

**Defaults are display-only until the user edits.** Mounting a dialog is not an
edit; `SchemaForm` fires no `onChange` on mount.

## Field identity

`FieldPath` is `(string | number)[]`. `pathKey` serialises it to a dot path
(`rules.0.attribute`), which is the key for three things: awareness focus,
error lookup, and the draft patches shared over Yjs.

Segments are escaped, because two things would otherwise read back as something
else:

| In a segment            | Encoded as | Why                                             |
| ----------------------- | ---------- | ----------------------------------------------- |
| `.`                     | `~1`       | A map key like `bldg.part` would split into two |
| `~`                     | `~0`       | So the markers are unambiguous                  |
| A string of only digits | `~2` + it  | A map key `0` would rebuild the map as an array |

Array indices stay bare digits, which is what keeps the Yjs wire format
unchanged. **Anything producing or consuming these keys must go through
`pathKey`/`parsePathKey`** — that includes `validate.ts`, which converts AJV's
JSON Pointer back, and `flattenObject` in `paramsAwareness.ts`.

Map keys are user-typed and the engine documents ones containing dots and
colons (`Attribute Table Extractor.inline` is keyed by names like
`bldg:Building`), so this is a real shape, not a hypothetical.

## Collaborative state

Two formats travel between clients, both in
`features/Editor/components/ParamsDialog/utils/paramsAwareness.ts`:

- **Draft patches** — `paramDrafts` in Yjs, keyed by dot path, one entry per
  edited field. Ordered by a **Lamport counter** (`seq`), not wall-clock time: a
  client whose clock runs fast would otherwise make its older edit overwrite a
  newer one. Merge order is `(seq, updatedAt, clientId)`; the client id is the
  final tie-break so every replica converges on the same result.
- **Awareness focus** — `focusedParamField`, the same dot path, keying
  `fieldFocusMap` so other users' focus can be drawn on the field.

## Errors

`ValidationErrors` is `Record<dotPath, string[]>`. **A key with an empty list is
a field that failed with nothing worth saying** — a required field left blank.
Its presence is what makes the field invalid; the list is only what to write
beneath it.

`useField` keeps the two questions apart: `errors` drives the text,
`hasErrors` drives the border and `aria-invalid`. Both follow the message, so a
required-but-blank field is marked by its asterisk alone. Colour is reserved for
a value that is actually wrong, where it points at something the field cannot
say for itself.

`SchemaForm` holds errors back until the user touches the form, so opening a
half-filled action does not greet them with errors they did not cause.

Two kinds of AJV noise are suppressed in `validate.ts`:

- `must be null` **from a confirmed branch of a choice** — `schemaPath` ending
  `/anyOf/<n>/type` or `/oneOf/<n>/type`. It fires whenever an optional section
  is present but incomplete, and the child's complaint is the useful one. It is
  a real error for a schema whose only permitted value is `null`, so the branch
  must be confirmed rather than assumed.
- `Choose one of the available options` where a more specific error already
  exists underneath it.

## Unions

`compile` reads the discriminator schemars emits — a required property pinned to
one value — and `selectVariant` matches on it directly, in priority order:
discriminator → bare constant → scalar type → required-key match. No scoring
branches by validation.

The discriminator is **dropped from the variant's rendered fields**, since the
dropdown already asks that question; it is re-attached to the data on change.
Switching variants carries over keys the target also has, and stashes the
outgoing variant's data so switching back is not destructive.

## Making changes

**A new field kind** — add the type to `types.ts`, classify it in `compile.ts`,
add a component under `fields/`, add the case to `fields/Field.tsx`. Handle it
in `defaults.ts` and `migrate.ts` if the value needs seeding or carrying.

**A new schema shape** — teach `normalize.ts` (if it is a combinator or
reference form) or `compile.ts` (if it is a new kind of value). Never widen
`corpus.test.ts` to make a failure go away; that gate is the thing keeping the
compiler honest.

**A new expression flavour** — `format: "code"` is how every expression field is
recognised. Python vs FlowExpr is not in the schema, so it comes from
`CompileOptions.pythonFields`, an explicit `(actionName, dot path)` table.

**Editor dialogs** — a `format: "code"` value is always `{ type, value }`, never
a bare string. Unwrap it for the editor and re-wrap on submit, preserving the
type the field already had.

## Testing

`yarn test --run` — the gates that matter:

| Test                                   | Guards                                                    |
| -------------------------------------- | --------------------------------------------------------- |
| `lib/schemaForm/corpus.test.ts`        | Every action compiles with **zero** `unsupported` nodes   |
| `lib/schemaForm/path.test.ts`          | Key round-trips, and no corpus property name contains `.` |
| `lib/schemaForm/migrate.test.ts`       | Migration never introduces an error an empty value lacks  |
| `lib/schemaForm/defaults.test.ts`      | Seeding never introduces an error either                  |
| `components/SchemaForm/index.test.tsx` | Every action renders; clearing, unions, optional sections |

The corpus gate reads `engine/schema/actions.json` directly, so a shape the
engine introduces that the UI cannot draw fails in the UI's CI on an
engine-only change — `engine/schema/**` is in the UI's change detection for
this reason.

## Gotchas

**`compile(null)`** is valid input. 30 of the engine's actions publish
`parameter: null`, and `buildNewCanvasNode` hands it straight through.

**Four actions have a union at the schema root** — Feature Reader, JSON
Fragmenter, XML Fragmenter, PLATEAU4.SolarPositionCalculator. Anything reading
`schema.properties` to decide something will be wrong for them: changing a
variant writes the whole params object at path `""`, which is a real path and
not "no path".

**The rich-text field skips an incoming value while it has focus**, so a
collaborator's edit does not move the caret mid-sentence. Blur re-runs the
synchronisation; without that the edit is never applied at all.

**The number field renders `""` while the entry is unparseable.** A
`type="number"` input reports `-` or `1.` as an empty value with
`validity.badInput` set, holding the characters in a buffer nothing can read.
Rendering anything else — the stored value, or a draft of the raw text — resets
the control and takes them with it.

**`onEditorOpen` / `ValueEditorDialog` is currently unreachable.** It required a
field typed `$ref: "#/definitions/Expr"`, which the engine no longer emits. The
prop is kept plumbed for when something needs it.
