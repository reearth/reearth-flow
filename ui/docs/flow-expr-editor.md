# FlowExpr Editor — Architecture

The FlowExpr editor is a custom code editor built from a plain `<textarea>` with layered overlays. It is **not** Monaco or CodeMirror.

## Files

| File                            | Role                                                                              |
| ------------------------------- | --------------------------------------------------------------------------------- |
| `FlowExprCodeEditor.tsx`        | Main component — composes all layers, manages validation debounce, scroll sync    |
| `FlowExprSyntaxHighlighter.tsx` | Hand-written tokenizer → colored `<span>` elements                                |
| `FlowExprAutocomplete.tsx`      | Dropdown positioned via canvas text measurement                                   |
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

`FlowExprAutocomplete.tsx` positions the dropdown by:

1. Finding the cursor word start/end
2. Measuring text width with a `canvas` element using the textarea's computed font
3. Combining that with `paddingLeft`, `lineHeight`, and `scrollTop` offsets

Autocomplete suggestions in `flowExprConstants.ts` use `{{cursor}}` as a placeholder in `insertText`. The editor replaces `{{cursor}}` with an empty string and positions the cursor at that index after insertion.

## Validator

`FlowExprValidator.ts` performs two checks only:

- **Bracket matching** — tracks `(`, `[`, `{` on a stack; reports unmatched or mismatched brackets
- **Unclosed strings** — detects `"` with no closing `"` on the same line (FlowExpr strings are single-line)

It does **not** type-check, evaluate, or validate identifiers — it cannot know the workflow context (feature attributes, env vars, available actions). Do not add semantic validation here.

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
