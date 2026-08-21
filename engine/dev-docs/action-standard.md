# Action Standard

Reference for authoring and reviewing Re:Earth Flow actions. Covers naming, descriptions, parameters, ports, categories, and tags.

---

## How to use this standard

This standard applies to both **authoring new actions** and **reviewing existing ones**. Use §8 as a self-check before submitting a new action, and as a review checklist during audits.

**Who the design is judged for.** Judge every action as a general-purpose geospatial tool, for a user who has never seen the project that motivated it. An action built for one dataset or one pipeline is still held to that bar. Low or zero usage in existing workflows is a reason to look *harder* — it often means the action is hard to understand or does not do the general thing its name promises — never a reason to skip the review.

**Prior art.** For anything with an established equivalent — a geometry operation, a file format's options, a well-known transformation — read how comparable tooling exposes it before designing names, parameters, or ports. That includes open specifications and implementations (OGC Simple Features, PostGIS, JTS, GDAL/OGR) and the commercial GIS tools users arrive from. The point is not to copy: it is that a user who already knows the operation should recognise ours, and that a difference should be a deliberate improvement rather than an accident. See §2 for how to cite prior art in text that ships.

**The schema is generated — never edit it directly.** All action properties (name, description, parameters, ports, categories, tags) are defined in the Rust implementation. After any change, regenerate the schema:

```bash
cargo make schema-base        # regenerates actions.json and syncs i18n skeletons
cargo make schema-translated  # regenerates per-language JSON files
```

**The implementation is the source of truth — verify against it first, every time.** Every user-facing property of an action must be traceable to code that delivers it. A name, title, or description must describe what the code actually does — not what the parameter name suggests, not what a prior description claimed, not what was intended.

Text that reads well is not evidence. A description can be fluent, accurate-sounding, and compliant with every rule below while describing behavior that does not exist, so **a property that looks correct is not exempt from being checked.** This is the most common way the surface starts lying: nobody writes an obviously wrong description, so the wrong ones are the ones that read best.

Whether authoring an action or reviewing one, before the schema is regenerated read the factory's `build`, the parameter struct, and the execution path (`process`/`start`/`run`), and confirm each of the following:

- **Every parameter is actually read and applied.** A parameter accepted but never used (e.g. stored into a field with a `_` prefix and never referenced) is a bug, not something to document — flag it for removal rather than writing a description for behavior that does not exist. Check for *forwarding*: a parameter copied into another struct in `build` and never read from there is still unused. And check **which build it is unused in** — a `cfg`-gated action can read a parameter in one geometry world and ignore it in the other, so establish whether it is dead everywhere, dead only in the shipped build (a migration artifact — leave it, it belongs to the migration's own cleanup), or dead only in the legacy build (live, keep it). `schema/actions.json` is generated from the default build, so anything appearing there is what users see today.
- **Enum variants behave as their names and descriptions claim** — trace each variant to its branch in the code.
- **Defaults, fallbacks, and "when omitted" behavior match the text** — confirm the actual default value and the code path taken when the parameter is absent.
- **The description reflects real behavior** — what the action consumes, what it emits, and any side effects — including where inputs come from (e.g. a path read from the incoming feature vs. a fixed parameter).

A description that reads well but misstates behavior is worse than no description. When the code and an existing description disagree, the code is the source of truth: fix the description (or fix the code and flag it), never copy the stale claim forward.

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

Verb-first, present tense, third-person singular — start directly with the verb, no subject.

- 1–2 sentences — prefer one; use two only when a single sentence would be genuinely unclear
- End every sentence with a period — required for consistent rendering across all supported languages
- Describes what the action does to data, not how it is implemented
- Does not mention port names or internal implementation details

**Naming other software.** Doc comments on factories, parameters and enum variants are compiled into `actions.json` and shipped to users in the UI and the generated docs — they are product copy, not code comments. Where naming prior art genuinely helps a reader, cite an open specification or implementation (OGC Simple Features, PostGIS, JTS, GDAL/OGR). Do not name a commercial product: describing our behaviour as matching, differing from, or replacing a named vendor's tool reads as a comparative claim we do not want to make or maintain, and it dates badly. Research those tools freely (§"Prior art") — just do not put their names in text that ships. This applies to `///` comments in particular, since it is easy to forget they are user-facing.

| ✗ | ✓ |
|---|---|
| "This processor calculates area" | "Calculates the planar or sloped area of polygon geometries." |
| "Uses the GEOS library to buffer geometries" | "Expands or contracts a geometry by a fixed distance." |
| "Routes to the `failed` port on error" | "Validates geometry against selected rules." |
| "Extract geometry parts as separate features" | "Extracts geometry parts from 3D geometries, emitting each as a separate feature." |

---

## 3. Parameters

### 3.1 Naming

- camelCase: `outputAttribute`, `targetEpsgCode`, `groupBy`
- No abbreviations except universally understood ones: `epsg`, `crs`, `url`, `id` are fine; `attr`, `cfg`, `val` are not
- No redundant type prefixes: `stringValue` → `value`

**Names must be accurate.** The accuracy rule that governs titles and descriptions applies to the parameter name itself: it must describe what the parameter actually controls, and it must not carry a meaning the implementation contradicts. Check the name against the term the operation is known by outside this project — OGC Simple Features, PostGIS, JTS — and prefer that term when one exists.

| ✗ | ✓ | Why |
|---|---|---|
| `unitSquareSize` | `cellSize` | a "unit square" has side 1, so `unitSquareSize: 5.0` contradicts itself; the established term is cell (or edge) size |
| `keepSquareOnly` | `completeCellsOnly` | every cell is a square — the real distinction is complete vs. partial |
| `mode` | `overlapBehavior` | names the mechanism, not what is being decided |

This is easy to miss because a name can satisfy every rule above and still be wrong. Read it the way a first-time user will, with no access to the implementation.

### 3.2 Required vs optional

- A parameter is **required** if the action cannot produce meaningful output without it — it must appear in the schema's `required` array
- A parameter is **optional** if the action can run sensibly without it — whether via a schema `default` or an implementation fallback, the action must never fail at runtime when an optional parameter is absent

### 3.3 Titles and descriptions

The parameter schema object itself must have a top-level `description` summarising what the parameter block configures. Every individual parameter property must have both a `title` (used as the UI field label) and a `description`.

- `title`: short noun phrase in title case — "Output Attribute", "Target EPSG Code"
- Prefer one sentence for `description`; two sentences are acceptable when the parameter behaviour is complex enough to warrant it
- Describes what the parameter controls and what values are valid
- Does not restate the parameter name or the action name: `"The outputAttribute"` or `"Geometry Splitter Parameters"` adds nothing
- For enums, describes what each variant does — see §3.4 for the mechanism and when each approach applies

### 3.4 Enum values

- camelCase: `planeArea`, `slopedArea`, `useAttributesFromOneFeature`
- No `SCREAMING_SNAKE_CASE`
- Values must be self-describing: `overwrite` not `1`, `skipExisting` not `0`

**Per-variant descriptions** are strongly preferred. Add them via doc comments on the Rust enum variant — schemars converts these into a `oneOf` entry with `title` and `description`:

```rust
enum AreaType {
    /// # Planar Area
    /// Calculates the flat projected area of the polygon.
    PlaneArea,
    /// # Sloped Area
    /// Calculates the true surface area accounting for slope.
    SlopedArea,
}
```

A plain `enum` with no doc comments produces no per-variant descriptions and should be converted to this pattern. A comprehensive property `description` that names and explains all variants is acceptable only when the enum has two or three self-describing values and the description remains one sentence.

**Keep mode enums inside a property — never make the parameter block itself a `oneOf`.** A Rust enum used *as the whole parameter type* (`#[serde(tag = "...")] enum FooParam`) generates a schema whose root is a `oneOf` rather than an object with `properties`. Translation cannot reach the variants: `apply_parameter_i18n` (`cli/src/utils.rs`) patches the root's own title/description, root `properties[*]`, `definitions[*].properties[*]`, and `definitions[*].oneOf|anyOf` variants — and that last traversal is scoped *inside* the `definitions` object, so a `oneOf` sitting at the schema root is never visited.

The failure is quiet, which is what makes it dangerous: the block's own title and description still translate via the root, so the action looks localised while the mode labels the user actually chooses between stay in English permanently. `Geometry Filter` is in this state today — its Japanese entry has a translated block header and no variant entries at all.

Give the action a normal parameter object with the enum as one property instead. When a mode needs its own sub-parameters, use a `#[serde(tag = "type")]` enum *as a property value* — the variants carry their own fields, the user only sees the fields belonging to the mode they chose, and the whole thing still translates.

**Single-variant enums** are a design smell — they present the user with a parameter that has no real choice. If only one variant exists and no others are planned, remove the parameter and hard-code the behavior. If additional variants are planned but not yet implemented, keep the `oneOf` and note the intent in a code comment (`// TODO: add X, Y variants`).

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

`default` is never a valid port name — always use one of the names below or a descriptive custom name.

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

- Validators and conditional routers must route every feature to a named port
- Actions with multiple semantically distinct outputs should use descriptive names rather than `features`

**Failure versus no-op — get this right before adding a `rejected` port.**

Ask: did the action *attempt* the transformation and fail, or was there simply *nothing to do*? Only the first is `rejected`.

| Situation | Port |
|---|---|
| Input was malformed — unparseable geometry, undecodable payload, a value of the wrong type | `rejected` |
| The action had nothing to act on — no geometry where geometry is optional, an absent attribute, a type the action legitimately leaves alone | pass through on `features` |

A missing attribute is **usually a no-op, not a failure.** Treating it as a failure is the most common way this rule gets broken, because "the attribute wasn't there" sounds like an error when it is normally just an absence.

**Check the paired producer.** Extractor/Replacer, Splitter/Merger and similar pairs only work if both halves agree. If the producer conditionally skips writing its attribute, the consumer *must* tolerate its absence — a `rejected` port on the consumer silently deletes the features the producer deliberately left alone.

**Adding a port is a data-loss change.** A new port is unwired in every existing workflow, so features newly routed to it are dropped on the floor rather than reaching the destination they used to. Before adding one: grep the fixtures for the action name to size the blast radius, and run `cargo make test-qc` afterwards. Silent loss shows up as a downstream count that quietly drops, not as a test error at the changed node.

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

Tags exist to cut **across** categories. The category is where a user browses; a tag is how they find the action from somewhere else. `citygml` earns its place because it spans `Input`, `Output`, `Geometry`, and `Filter` — one tag collects the whole CityGML toolkit. `geometry` on a `Geometry` action tells the user nothing they did not already know from where they found it.

- All lowercase, hyphenated if multi-word: `coordinate-system`, `citygml`
- **The test:** would this tag help someone find the action from *outside* its category? If not, it is padding.
- **Zero tags is a valid and complete answer.** When every candidate would restate the category, the action has no cross-cutting axis and takes none. This is normal for general-purpose processors — most `Geometry` and `Debug` actions are in this position. Never invent a tag to avoid an empty list.
- Two to four tags where the axes genuinely exist — typically format, dimensionality, or domain. Readers and writers almost always qualify (`csv`, `gltf`, `geopackage`, `citygml`); a plain geometry operation usually does not.
- **Check the siblings.** An action doing the same kind of work as a tagged action should carry the same tag. A gap between siblings — `Three Dimension Forcer` tagged `3d` while `Two Dimension Forcer` has nothing — is an oversight, not a judgement, and it is the reliable way to tell the two apart.
- Draw from the established vocabulary below; propose additions conservatively

**Established vocabulary:**
`3d`, `aggregation`, `attribute`, `citygml`, `compression`, `coordinate-system`, `csv`, `database`, `debug`, `excel`, `file`, `filter`, `geometry`, `geojson`, `geopackage`, `gltf`, `json`, `list`, `logging`, `mapping`, `obj`, `raster`, `routing`, `scripting`, `shapefile`, `spatial`, `statistics`, `tiling`, `validation`, `vector`, `xml`

Some vocabulary entries share a name with a category (`geometry`, `filter`, `file`, `debug`, `attribute`). Those are valid only *outside* the matching category, where they still cut across — `geometry` on `Dimension Filter` and `file` on `Zip File Writer` both earn their place; `file` on a `File` action does not.

New tags can be proposed when an established term does not adequately describe an action's domain.

---

## 7. Functionality and exposure

Every other section in this standard asks whether an action is *described* correctly. This one asks whether it *works*, and whether it should be offered at all. An action can satisfy every rule above — clean name, accurate parameters, complete ports — and still be broken or inappropriate to ship, so these are checked separately.

### 7.1 Does it run?

Confirm the action executes in the build that ships. Two ways it can fail to:

- **Trait defaults.** `Processor::process` and `Source::start` have defaults that return ``Err("`{name}` is not yet ported to new geometry")``. An action whose only implementation is under `#[cfg(not(feature = "new-geometry"))]` therefore fails on **every feature** in the shipped build, while looking complete in the source.
- **Deliberate stubs.** A `process` that returns `Err` unconditionally — typically an action superseded by another — is a removal that has not happened yet, not a working action.

Check by confirming an implementation exists for the default feature set, not merely that one exists.

### 7.2 Should it be exposed?

`server/api/internal/app/base_actions.go` gates which actions appear in the palette. It is a separate file in a separate language from the action itself, so the two drift silently. An action must not be listed there unless it runs per §7.1.

This cuts both ways, and both have happened:

- Promoting an action without an engine-side review ships whatever state its metadata was in.
- Leaving a listed action to rot means the palette advertises something that always fails.

When an action cannot run, removing it from `base_actions.go` is the immediate mitigation — cheap, reversible, and independent of whatever fix is pending.

### 7.3 Disabled behaviour must not stay documented

If behaviour is turned off — a code path commented out, a branch rerouted, functions kept only under `#[allow(dead_code)]` — then the parameters, enum variants, and description advertising it are now false and must go with it. "Temporarily disabled" is not a state the user-facing surface can represent: from the outside there is no difference between a feature that is off and one that never worked.

Silencing a warning is not a substitute for either finishing or removing the work. `#[allow(dead_code)]` on an action code path is a review flag: it means something is unreachable, and unreachable code with live documentation is exactly how the surface starts lying.

---

## 8. Review Checklist

For each action, flag anything that violates the rules above. Only log issues — skip clean items.

**First, verify against the implementation** (see "How to use this standard"): read the factory and execution path and confirm every parameter is actually used, enum variants and defaults behave as documented, and each title/description matches real behavior. Accuracy is checked before style — a well-worded but incorrect description is a defect, and a parameter that is accepted but never applied is flagged for removal, not documented.

**Check that it runs before reviewing how it reads** (§7.1). Polishing the description of an action that cannot execute is wasted work, and it makes a broken action look reviewed.

```
ActionName
  runs:    [only if it does NOT run in the shipped build, or is listed in
              base_actions.go while broken (§7)]
  impl:    [parameters declared but never applied; enum variants with no branch;
              defaults or "when omitted" text the code contradicts; declared
              ports never emitted]
  name:    [proposed space-case name if different]
  desc:    [issue if any]
  params:  [list issues by param name; flag if count exceeds 8 without justification (§3.5)]
  ports:   [failure-vs-no-op errors (§4.3); missing or unemitted ports]
  cat:     [issue if wrong category]
  tags:    [missing cross-cutting tag | tag that restates the category (§6)]
```

**Every line except `impl:` can be answered from the generated `actions.json`. `impl:` cannot.** It is the only one that requires opening the code, and it is therefore the only one that catches a description which is well-written and false, or a parameter the UI offers and the code ignores. An action marked clean without `impl:` having been worked through has been *read*, not checked.

If an action is clean on all dimensions, write: `ActionName — OK`

**Deferred findings need a stated trigger.** When an item is parked because something else blocks it, record what would unblock it, in the finding itself — "revisit when X is ungated", not "deferred for now". A deferral whose precondition silently expires is indistinguishable from a finding nobody wrote down. This has already cost us: the reprojector removal was correctly deferred while its replacement was gated out of the shipped build, the gate later lifted, and nothing reopened the item.

---

## Changelog

Material rule changes, newest first. **A rule added here does not retroactively apply to actions already reviewed** — when a change would alter a past verdict, say so in the entry, and treat previously-reviewed actions as owing a re-check against the new rule.

### 2026-08-21 (later)

- **§6** — added `excel` to the established tag vocabulary. Every audited reader and writer carries its format as a tag (`csv`, `json`, `geojson`, `shapefile`, `geopackage`), and `Excel Writer` had no term to draw on. No past verdict changes.

### 2026-08-21

- **§"How to use" rescoped — this widens what the rule covers.** The verify-against-implementation duty was previously worded as a precondition for *adding or editing* a title or description, which exempted any property that already looked compliant. It now attaches to the action itself: every user-facing property must be traceable to code that delivers it, checked before the schema is regenerated, whether authoring or reviewing. Reviews done under the old wording may have triaged on how the text reads rather than on what the code does.
- **§8** — added an `impl:` line to the checklist, for parameters never applied, enum variants with no branch, defaults the code contradicts, and ports never emitted. Every other line in the checklist can be answered from the generated schema; this one cannot, which is the point.
- **§6 corrected — this reverses previous guidance.** The old wording set a floor ("aim for 2–4; 1 is acceptable") while also forbidding tags that duplicate the category, which made the floor unreachable for single-domain actions and invited padding. Tags are now defined by their purpose — cross-category discovery — and **zero tags is explicitly valid** when no orthogonal axis exists. Actions reviewed under the old wording may carry a tag that only restates the category, and a zero-tag action is no longer a finding on its own. Sibling comparison is the test for a genuine omission.

### 2026-08-20

- **§4.3 corrected — this reverses previous guidance.** The old wording listed "missing attribute" as a `rejected` case. That is wrong: a missing attribute is normally a no-op that must pass through, and the failure-versus-no-op distinction now has its own subsection. Actions reviewed before this date may carry a `rejected` port added on the old advice; those are worth re-checking, particularly either half of a producer/consumer pair.
- **§7 added** (Functionality and exposure), covering whether an action runs in the shipped build, whether `base_actions.go` should list it, and the rule that disabled behaviour cannot stay documented. No earlier review checked these, so every action reviewed before this date is unverified on them. The review checklist gains a `runs:` line.
- **§3.4** — added the prohibition on making a parameter block a root-level `oneOf`, which permanently blocks translation. Existing actions shaped this way need restructuring, not just re-wording.
- **§"How to use"** — the unused-parameter rule now requires establishing *which* geometry world a parameter is dead in, and catching forwarding-then-dropped parameters.
- **§"How to use"** — added the audience statement and the prior-art requirement.
- **§2** — added the rule against naming commercial products in text that ships.

### 2026-07-23

- Added the ACCURACY-BEFORE-STYLE clause in §"How to use" (PR #2280). Introduced mid-audit: the Debug, Merge, Filter and Output batches were reviewed before it existed and were never re-checked against it.
