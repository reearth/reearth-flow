# CityGML schema validation

Tooling that validates the writer's output against the vendored OGC CityGML 2.0 schemas, offline.

The schemas live in `runtime/citygml/schemas/`, embedded into the engine by `reearth-flow-citygml`'s `build.rs` and listed in its `MANIFEST.tsv`. The catalog here points `xmllint` at that same copy. The writer reads the schemas' content rules at runtime through `reearth_flow_citygml::schema`, so there is no generated table to regenerate.

`libxml2` speaks HTTP but not TLS, and `schemas.opengis.net` is HTTPS-only, so
`xmllint` cannot fetch these at run time. Committing the closure is the only way
to validate, and it also keeps CI off the network.

- `citygml-2.0.xsd` imports every module the writer can emit, so one `--schema`
  argument covers any document it produces.
- `catalog.xml` maps the OGC, W3C and OASIS URLs onto the schemas. Prefixes are
  relative to the catalog, so the tree works from any checkout.
- `known-invalid.txt` lists cases that do not validate yet.

Run with `cargo make check-citygml-schema`.
