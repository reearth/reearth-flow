# FlowExpr Language Specification

This directory is the authoritative definition of the FlowExpr language.

## Stabilization Process

Everything documented here is stable for the current language type (`flowExpr`).
Breaking changes require a migration tool.
Semantic breaking changes introduce a new language type (`flowExpr2`, …) rather than modifying existing behavior.

Writing the spec is the stabilization decision.
The implementation must be fully compatible with this spec.
The implementation may contain unstabilized features not yet documented here.

## Documentation Hierarchy

Spec files follow the naming convention `<number>-<name>.md` (e.g. `1-base-language.md`).
They are ordered by the time the decision was proposed, not by topic.

Other documentation structured by topic can be generated from these files as the authoritative source.
