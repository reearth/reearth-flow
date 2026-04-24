---
name: add-action
description: Step-by-step guide for adding or modifying a Rust action in the engine. Use when creating a new SourceFactory, ProcessorFactory, or SinkFactory, registering an action, or updating action schemas and i18n files.
user-invocable: true
---

# Add a New Engine Action

Follow these steps when adding or modifying an action in the engine.

## Steps

1. Create a factory struct implementing the appropriate trait (`SourceFactory`, `ProcessorFactory`, or `SinkFactory`)
2. Define the parameter struct with `serde` + `schemars` derives — use `#[schemars(title = "...", description = "...")]` on fields for English display strings
3. Implement the `build()` method returning the action instance
4. Register the factory in the appropriate mapping file
5. Run `cargo make schema-base` — regenerates `actions.json` and syncs i18n skeleton entries for the new action across all language files
6. Fill in translated strings in `schema/i18n/actions/{lang}.json` for each language
7. Run `cargo make schema-translated` — generates all `actions_{lang}.json` files and docs

**Always run both commands in order. Never run `schema-translated` without first running `schema-base` when action code has changed.**

## Action i18n

Translated action schemas are generated from source files in `schema/i18n/actions/{lang}.json`. **Never edit the generated `schema/actions_*.json` files directly** — they are always overwritten by `cargo make doc-action`.

Each i18n entry supports:

```json
{
  "name": "MyAction",
  "description": "Translated action description",
  "parameterI18n": {
    "someProperty": { "title": "Translated title", "description": "Translated description" }
  },
  "definitionI18n": {
    "SomeDefinition": {
      "fieldName": { "title": "Translated title" }
    }
  }
}
```

- `parameterI18n` — keys are top-level parameter property names (from `schema["properties"]`)
- `definitionI18n` — keys are definition names (from `schema["definitions"]`), values are maps of property name → i18n
- Both fields are optional; missing or empty values fall back to the English strings from schemars annotations
- Property names come directly from Rust field names after camelCase conversion (predictable without running the generator)
- `cargo make schema-base` reconciles all lang files as part of its run: adds missing keys, removes stale keys, preserves existing translations
