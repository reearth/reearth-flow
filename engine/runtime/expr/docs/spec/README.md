# FlowExpr Language Specification

This directory is the authoritative definition of the FlowExpr language.

## Stabilization Process

Everything documented here is stable for the current language type (`flowExpr`).

Writing the spec is the stabilization decision.
The implementation should be compatible with this spec.
The implementation may contain unstabilized features not yet documented here.

When one spec subsume another, the more general one wins.
For example, if one spec says foo() takes int, and another says foo() takes int and string, the second spec wins.
If two specs are not compatible, it is considered a bug.

Spec should describe the language design as an interface defined by behavior, not its implementation.

## Documentation Hierarchy

Spec files follow the naming convention `<number>-<name>.md` (e.g. `1-base-language.md`).
They are ordered by the time the decision was proposed, not by topic.

Other documentation structured by topic can be generated from these files.

Code should refer spec only, not user-facing documentation.
Refer spec section from code with anchors following the GitHub convention: convert section title to lowercase, spaces replaced with `-`.

## Writing Style Guideline

- Sections should be atomic and describe one point, not a group of points - use child sections for that.
- Avoid duplicating existing points. Use cross-reference, especially when new spec extends previous spec's some section.