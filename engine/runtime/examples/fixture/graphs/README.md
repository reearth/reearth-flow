# graphs

Shared subgraphs that workflows pull in with `!include`. A subgraph read by only one workflow does not belong here — it lives next to that workflow's `workflow.yml`.

## Placement rules

1. `graphs/` itself holds only generation-independent graphs, i.e. ones actually read by workflows of more than one PLATEAU generation.
2. `graphs/plateau4/` and `graphs/plateau6/` hold graphs shared by several workflows of that generation. Any graph whose action names carry a `PLATEAU4.` / `PLATEAU6.` prefix belongs here.
3. `graphs/plateauN/quality-check/` holds exactly two files: `01-01-common.yml` and `01-02-common.yml`. It is the place for the building blocks of the 01-common workflow, not a general shared-graph directory. Graphs shared across feature types go directly under `graphs/plateauN/`.
4. A subgraph read by a single workflow stays beside that workflow's `workflow.yml`.

## Layout

```
graphs/
├── lod_splitter.yml                     generation-independent
├── plateau4/
│   ├── xml_validator.yml
│   ├── domain_of_definition_validator.yml
│   ├── folder_and_file_path_reader.yml
│   ├── lod_splitter_with_dm.yml
│   ├── surface_validator.yml
│   └── quality-check/
│       ├── 01-01-common.yml
│       └── 01-02-common.yml
└── plateau6/
    ├── xml_validator.yml
    ├── domain_of_definition_validator.yml
    ├── surface_checks.yml
    └── quality-check/
        ├── 01-01-common.yml
        └── 01-02-common.yml
```

## Who reads what

- `lod_splitter.yml` — quality-check 02-bldg and 03-tran, in both plateau4 and plateau6
- `plateau4/xml_validator.yml`, `plateau4/domain_of_definition_validator.yml`, `plateau4/quality-check/01-01-common.yml`, `plateau4/quality-check/01-02-common.yml` — every quality-check plateau4 workflow
- `plateau4/folder_and_file_path_reader.yml` — every data-convert plateau4 workflow, plus `workflow/examples/citygml-roundtrip` and `workflow/solar-radiation`
- `plateau4/lod_splitter_with_dm.yml` — 7 data-convert plateau4 workflows and 8 quality-check plateau4 workflows
- `plateau4/surface_validator.yml` — 8 quality-check plateau4 workflows
- `plateau6/xml_validator.yml`, `plateau6/domain_of_definition_validator.yml`, `plateau6/quality-check/01-01-common.yml`, `plateau6/quality-check/01-02-common.yml` — every quality-check plateau6 workflow
- `plateau6/surface_checks.yml` — quality-check plateau6 02-bldg and 03-tran

## Notes

- `plateau4/lod_splitter_with_dm.yml` and `plateau4/surface_validator.yml` keep their graph names `PLATEAU3.LodSplitterWithDM` and `PLATEAU3.SurfaceValidator3D`. Those names are also baked into the flattened single-file workflows, so the directory and the name disagree on generation on purpose.
- Everything under `graphs/plateau6/` assumes new-geometry.
