# CityGML schema validation

Vendored OGC CityGML 2.0 schemas and the XML catalog that resolves them offline.

`libxml2` speaks HTTP but not TLS, and `schemas.opengis.net` is HTTPS-only, so
`xmllint` cannot fetch these at run time. Committing the closure is the only way
to validate, and it also keeps CI off the network.

- `citygml-2.0.xsd` imports every module the writer can emit, so one `--schema`
  argument covers any document it produces.
- `catalog.xml` maps the OGC, W3C and OASIS URLs onto `schemas/`. Prefixes are
  relative to the catalog, so the tree works from any checkout.
- `known-invalid.txt` lists cases that do not validate yet.

Run with `cargo make check-citygml-schema`.
