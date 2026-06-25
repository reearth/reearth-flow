# FlowExpr Language Specification

This directory is the authoritative definition of the FlowExpr language.

## Stabilization Process

Writing the spec is the stabilization decision.
The implementation should be compatible with this spec.
The implementation may contain unstabilized features.

Spec should describe the language design as an interface defined by behavior, not its implementation.

## Documentation Hierarchy

Spec files follow the naming convention `<number>-<name>.md` (e.g. `1-base-language.md`).
They are ordered by the time the decision was proposed, not by topic.

Other documentation structured by topic can be generated from these files.

Code should refer to the spec only, not user-facing documentation.
Refer to spec sections from code with anchors following the GitHub convention: convert section title to lowercase, spaces replaced with `-`.

## Writing Style Guideline

- Do not mix multiple language features into one section. For multiple related features, use subsections.
- Each section should be self-contained. Rationale, exceptions, and history belong in the section body, not as subsections.
- Avoid rewording existing points. Use cross-reference, especially when new spec extends some previous spec's section.
- Do not over-specify. Simply do not mention a behavior if it is undefined. For example:
    "`+` adds two numerical values, and errors on other types" is bad since it breaks extensibility forever.
    The better style is to simply not mention the error behavior, or specify the exact types that should error.