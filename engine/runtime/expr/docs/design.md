# expr design notes

This document records non-obvious design decisions for cross-referencing from source comments.

DEPRECATED: use comments directly if it is implementation specific; use spec if it is a language feature.

## No while-loop iteration limit {#no-while-iteration-limit}

Reviewers sometimes suggest capping the number of `while` iterations as a safety measure. This is intentionally not done, for several reasons:

1. **Inconsistency**: capping `while` but not `for` is arbitrary; capping `for` makes no sense either because it iterates over an explicit collection whose size is already determined by the data — users do not want an artificial cap on that.
2. **Wrong abstraction level**: workflow authors often cannot predict input scale, so any hard number will either be too small for legitimate workloads or too large to provide real protection.
3. **Limited benefit**: an ill-formed expression that loops forever produces no useful output regardless — an early stop does not help the workflow recover. Action-level timeouts in the executor framework are the correct and consistent place to enforce wall-clock limits.
4. **Turing-completeness expectation**: once a language has unbounded loops, adding a hidden iteration cap is surprising and breaks reasonable user expectations.

## `find()` null-as-falsy is intentional; empty-string matches indicate a broken pattern {#regex-find-null-falsy}

`find()` returns `null` on no match, so callers can use it directly as a boolean condition. The edge case of a pattern that matches an empty string (e.g. `".*"`) produces a falsy result even though technically something was "found". This is intentional:

- If the pattern has a capture group that captures an empty string, the pattern matches every input — that is a malformed pattern, not a legitimate result.
- If the pattern has no capture group and the full match is empty, the input string itself is empty, and a validator that "passes" on an empty input is again a malformed pattern.

In both cases the empty match signals a broken pattern, and returning a falsy value is the correct behavior. Users who need an explicit boolean should write `find() != null`, which also handles this case correctly.
