# Embedded schema notices

`reearth-flow-citygml` embeds these XML Schema documents so the CityGML Writer
can read the content rules of CityGML and of the i-UR ADE at runtime without
network access. Each is
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

## i-UR 3.0, 3.1 and 3.2 (`www.geospatial.jp/iur/schemas/`)

The i-UR (i-Urban Revitalization) ADE schemas, `uro/<version>/urbanObject.xsd`
and `urf/<version>/urbanFunction.xsd`, are published by the Secretariat for the
Promotion of Regional Revitalization, Cabinet Office, Government of Japan, on the
i-UR Technical Materials page
(https://www.chisou.go.jp/tiiki/toshisaisei/itoshisaisei/iur/index.html), which
links each schema at the URL in `MANIFEST.tsv`. The files carry no licence
statement, and neither does that page, so the site's terms of use
(利用規約, https://www.chisou.go.jp/sousei/terms.html) apply: copyright in the
site's content belongs to the Cabinet Office (内閣府) unless stated otherwise,
and content without a rights notice is provided under the Public Data License
1.0.

**Licence:** Public Data License 1.0 (公共データ利用規約 第1.0版, PDL 1.0),
https://www.digital.go.jp/resources/open_data/public_data_license_v1.0 . It
permits copying and redistribution with a source credit and is compatible with
CC BY 4.0. The credit required by the licence:

> 出典：内閣府地方創生推進事務局「i-都市再生技術仕様（案）/i-UR Technical Materials」
> (https://www.chisou.go.jp/tiiki/toshisaisei/itoshisaisei/iur/index.html)

## W3C: XLink and XML (`www.w3.org/`)

The files under `www.w3.org/` carry no licence statement. They are published by the W3C at the
URLs in `MANIFEST.tsv`.
