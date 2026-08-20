# FlowExpr Editor — Architecture

The FlowExpr editor is a custom code editor built from a plain `<textarea>` with layered overlays. It is **not** Monaco or CodeMirror.

## Files

| File                            | Role                                                                              |
| ------------------------------- | --------------------------------------------------------------------------------- |
| `FlowExprCodeEditor.tsx`        | Main component — composes all layers, owns caret state, validation debounce       |
| `FlowExprSyntaxHighlighter.tsx` | Hand-written tokenizer → colored `<span>` elements                                |
| `FlowExprAutocomplete.tsx`      | Dropdown positioned via canvas text measurement                                   |
| `flowExprAttributeContext.ts`   | `getCompletionContext` — what is being completed at the caret                     |
| `FlowExprValidator.ts`          | Client-side bracket matching + unclosed string detection                          |
| `flowExprConstants.ts`          | Keywords, built-in functions, math functions, operators, autocomplete suggestions |
| `constants.ts`                  | Shared `AutocompleteSuggestion` type                                              |

All files live under:
`src/features/Editor/components/ParamsDialog/components/ValueEditorDialog/components/`

## Overlay z-index stack

Four layers are stacked with absolute positioning:

| z-index | Layer           | Purpose                                                                         |
| ------- | --------------- | ------------------------------------------------------------------------------- |
| 1       | Highlight div   | Syntax-colored spans (pointer-events: none)                                     |
| 3       | Textarea        | Transparent text, visible caret/selection                                       |
| 4       | Error overlay   | Underline spans for validation errors (pointer-events: auto for hover tooltips) |
| 0       | Placeholder div | Gray placeholder text when value is empty                                       |

The textarea text is `color: transparent` so the highlight layer shows through. The caret stays visible because it is rendered by the browser independently of text color. Scroll position is kept in sync between textarea and the highlight/error layers via `onScroll`.

## Syntax highlighter

`FlowExprSyntaxHighlighter.tsx` is a single-pass character scanner. Token priority order:

1. Whitespace
2. Double-quoted strings (`"…"`) — single quotes are **not** supported by FlowExpr
3. Numbers (integer and float)
4. Multi-character operators (longest-match, re-sorted on each render from `FLOWEXPR_OPERATORS`)
5. Punctuation `( ) { } [ ] ; , .`
6. Identifiers — classified as `keyword`, `function`, or `identifier` via array lookup; `math` followed by `::` becomes `namespace` + `operator`

The `math::fnName` tokens are classified as `namespace` + `operator` (`::`) + `identifier` — individual math function names are **not** classified as `function` tokens.

## Autocomplete

### Completion context

`getCompletionContext(text, caret)` in `flowExprAttributeContext.ts` is the single source of truth for what is being completed. It returns:

- `kind` — `attribute` when the caret is inside an open `attributes["…` accessor, otherwise `general`
- `prefix` — the text used to filter, taken **only from before the caret**
- `start` / `end` — the span a chosen suggestion replaces. `end` runs to the end of the token (the closing quote for attributes, the end of the identifier segment otherwise), so accepting mid-word overwrites the rest of the word instead of duplicating it

Filtering and insertion both read this one object, so the list can never be matched against text the insertion won't touch. In attribute mode the name is delimited by the quote rather than by word characters, so names containing spaces or hyphens complete correctly.

### Visibility

The editor owns two pieces of state: `caret` (mirrored from the textarea so context recomputes when the caret moves, not just when text changes) and `autocompleteArmed`.

- **Armed** by any edit — which is why backspacing back to a matching prefix brings suggestions back, and why a prefix that matches nothing is not a dead end
- **Disarmed** by Escape, by moving the caret without editing (arrow keys, clicks), and by accepting a suggestion
- Accepting a suggestion that leaves the caret inside an `attributes["…"]` accessor re-arms, chaining the accessor completion straight into the attribute list. Accepting an attribute _name_ is excluded, since the caret ends up in the same place and the list would never close

The dropdown renders only when armed **and** there are matching suggestions. Suggestions are derived with `useMemo` rather than stored in state, so recomputing them cannot schedule a render — an unrelated re-render of the Editor can no longer reset the highlighted item. `selectedIndex` resets only when the offered labels actually change.

`readerAttributeSuggestions` (`usePreviewSchema/index.ts`) is keyed on its own contents because `rawWorkflows` is rebuilt from the yjs doc on every render; without that, the prop identity changed constantly.

### Keyboard

Arrow/Enter/Tab are handled from the **textarea's own** `onKeyDown`, delegated to the dropdown through a ref handle that returns whether it consumed the key. It claims keys only while something is shown, so Enter/Tab/arrows elsewhere in the dialog keep working. Escape is the exception: it needs a capture-phase document listener to beat the Dialog's own handler, and that listener is mounted only while the dropdown is visible so Escape can still close the dialog otherwise.

### Positioning

The dropdown is placed by measuring the text before `context.start` with a `canvas` element using the textarea's computed font, combined with `paddingLeft`, `lineHeight`, and scroll offsets. It re-measures on textarea scroll. Soft-wrapped lines are not accounted for.

Suggestions in `flowExprConstants.ts` use `{{cursor}}` as a placeholder in `insertText`. The editor strips it and parks the caret at that index after insertion — applied in a layout effect once the controlled value lands, since React otherwise leaves the caret at the end of the textarea.

## Validator

`FlowExprValidator.ts` performs two checks only:

- **Bracket matching** — tracks `(`, `[`, `{` on a stack; reports unmatched or mismatched brackets
- **Unclosed strings** — detects `"` with no closing `"` on the same line (FlowExpr strings are single-line)

It does **not** type-check, evaluate, or validate identifiers — it cannot know the workflow context (feature attributes, workflow variables, available actions). Do not add semantic validation here.

Validation runs on a 300 ms debounce after each change.

## Keeping constants in sync with the engine

Always read the engine source directly before editing `flowExprConstants.ts` — the markdown reference doc can lag behind the implementation:

| What to check                                  | Engine file                                                |
| ---------------------------------------------- | ---------------------------------------------------------- |
| Keywords and operators                         | `engine/runtime/expr/src/core/lexer.rs` — the `Token` enum |
| Built-in functions (`str`, `int`, `Url`, etc.) | `engine/runtime/expr/src/core/eval.rs` — `default_env()`   |
| Math functions and constants                   | `engine/runtime/expr/src/core/builtins/`                   |

After reading the source, update **all five** in `flowExprConstants.ts`:

1. `FLOWEXPR_KEYWORDS` — control-flow keywords, boolean/null literals
2. `FLOWEXPR_BUILTIN_FUNCTIONS` — classified as `function` token type by syntax highlighter
3. `FLOWEXPR_MATH_FUNCTIONS` — reference list (not used by syntax highlighter directly)
4. `FLOWEXPR_OPERATORS` — keep sorted longest → shortest within each group
5. `getFlowExprAutocompleteSuggestions` — one entry per keyword/function/constant; include `detail` signature and `{{cursor}}` placement
