# Embedded schema notices

`reearth-flow-citygml` embeds these XML Schema documents so the CityGML Writer
can read CityGML's content rules at runtime without network access. Each is
unmodified from its source, listed with its URL in `MANIFEST.tsv`.

## OGC: CityGML 2.0 and GML 3.1.1 (`schemas.opengis.net/`)

Every file under `schemas.opengis.net/` carries the statement below, except the
SMIL copies under `schemas.opengis.net/gml/3.1.1/smil/` (W3C, next section) and
the xAL copy under `schemas.opengis.net/citygml/xAL/` (OASIS, see below):

> Copyright (c) Open Geospatial Consortium.
> To obtain additional rights of use, visit http://www.opengeospatial.org/legal/ .

The year varies by file; each file's own header is authoritative.

## W3C: SMIL 2.0 (`schemas.opengis.net/gml/3.1.1/smil/`)

`smil20.xsd` and `smil20-language.xsd` carry the statement:

> Copyright: 1998-2001 W3C (MIT, INRIA, Keio), All Rights Reserved.

## OASIS: xAL 2.0 (`docs.oasis-open.org/`, and the copy at `schemas.opengis.net/citygml/xAL/`)

Both `docs.oasis-open.org/election/external/xAL.xsd` and
`schemas.opengis.net/citygml/xAL/xAL.xsd` carry the statement:

> Copyright(c) 2000, OASIS. All Rights Reserved [http://www.oasis-open.org]

## W3C: XLink and XML (`www.w3.org/`)

The files under `www.w3.org/` carry no licence statement. They are published by the W3C at the
URLs in `MANIFEST.tsv`.
