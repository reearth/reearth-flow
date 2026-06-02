# Action Review Findings

Phase 3 quality review of the 74 base actions against [action-standard.md](action-standard.md).

**How to use:**
- Fill each action with either `ActionName — OK` or the checklist format from §7 of the standard
- Phase 4 improvement PRs should reference this file and delete completed sections as fixes land
- File is deleted when all sections are cleared

---

## Debug (5)

<!-- Session 1 -->

```
EchoProcessor
  name:    → "Echo Processor"
  desc:    not imperative — "Debug Echo Features to Logs"; suggest "Echo features to logs and pass them through unchanged."
  tags:    empty — `debug` duplicates category (§6); no other established vocabulary terms apply; consider proposing `logging`

EchoSink
  name:    → "Echo Sink"
  desc:    not imperative — identical to EchoProcessor; suggest "Echo features to logs and discard them."
  tags:    empty — same constraint as EchoProcessor

FeatureCounter
  name:    → "Feature Counter"
  params:  countStart — marked required but has a sensible default (0 or 1); should be optional with a schema default (§3.2)
           outputAttribute — title "Output Attribute" is generic; suggest "Count Attribute" (§3.3)
           ordering — `groupBy` (optional) is defined between two required params; once countStart is made optional, correct order: outputAttribute → countStart → groupBy (§3.5)
  tags:    empty — suggest ["aggregation", "attribute"]; `debug` duplicates category (§6)

NoopProcessor
  name:    → "Noop Processor"
  desc:    noun phrase — "No-Operation Pass-Through Processor"; suggest "Pass features through unchanged."
  tags:    empty — `debug` duplicates category (§6); no other established vocabulary terms apply

NoopSink
  name:    → "Noop Sink"
  desc:    noun phrase with parenthetical — "No-Operation Sink (Discard Features)"; suggest "Discard all incoming features."
  tags:    empty — same constraint as NoopProcessor
```

---

## Input (10)

<!-- Session 2 -->

---

## Output (10)

<!-- Session 3 -->

---

## Attribute (8)

<!-- Session 4 -->

---

## Filter (7)

<!-- Session 5 -->

---

## Merge (3)

<!-- Session 5 -->

---

## Feature (1) · File (2) · Transform (5)

<!-- Session 6 -->

---

## Geometry A (12)

<!-- Session 7 — AppearanceRemover through ImageRasterizer -->

---

## Geometry B (11)

<!-- Session 8 — PolygonNormalExtractor through VerticalReprojector -->
