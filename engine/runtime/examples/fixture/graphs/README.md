# graphs

Subgraphs that more than one workflow pulls in with `!include`. A subgraph read by a single workflow stays beside that workflow's `workflow.yml`.

## Placement

```
graphs/
├── <name>.yml                read across PLATEAU generations
├── plateau4/
│   ├── <name>.yml            read by several plateau4 workflows
│   └── quality-check/
│       └── *-common.yml      parts of the 01-common workflow only
└── plateau6/
    └── (same layout)
```

- A graph whose action names carry a `PLATEAU4.` / `PLATEAU6.` prefix belongs under that generation's directory (e.g. `plateau4/xml_validator.yml`).
- `quality-check/` is not a general shared directory: it holds only the pieces of the 01-common workflow. Graphs shared across feature types go directly under `plateau4/` or `plateau6/`.

## Notes

- A graph name may name an older generation than its directory (e.g. `PLATEAU3.LodSplitterWithDM` under `plateau4/`). The names are baked into the flattened single-file workflows, so they are kept as-is.
- Everything under `plateau6/` assumes new-geometry.
