# Bundled XML Schema Definitions

This directory contains bundled XML Schema Definition (XSD) files used for offline validation of XML documents. These schemas are cached locally to avoid network latency and HTTP 429 errors when fetching from remote servers during validation.

## Purpose

When validating XML documents (such as CityGML files), schema validators typically fetch referenced schemas over HTTP. This can cause:
- Network latency and performance issues
- HTTP 429 (Too Many Requests) errors from public schema repositories
- Validation failures in offline or restricted network environments

By bundling these commonly-referenced schemas, we ensure reliable and fast validation.

## Included Schemas

### W3C Schemas (`w3c/`)

Standard W3C schemas for XML namespaces and linking:

- **xml.xsd** - XML namespace attributes (xml:lang, xml:space, xml:base, xml:id)
  - Namespace: `http://www.w3.org/XML/1998/namespace`
  - Source: [W3C XML Namespace](https://www.w3.org/2001/xml.xsd)

- **xlink.xsd** - XLink attributes for hyperlinks in XML
  - Namespace: `http://www.w3.org/1999/xlink`
  - Source: [W3C XLink 1.0 Recommendation](http://www.w3.org/TR/2001/REC-xlink-20010627/)

### OASIS Schemas (`oasis/`)

Address formatting standards:

- **xAL.xsd** - eXtensible Address Language
  - Namespace: `urn:oasis:names:tc:ciq:xsdschema:xAL:2.0`
  - Version: 2.0
  - Source: [OASIS Customer Information Quality TC](https://www.oasis-open.org/committees/ciq)
  - Used in: GML, CityGML address representations

## License Information

Each schema directory contains a LICENSE file with the applicable terms:

- **W3C schemas**: [W3C Software and Document License](https://www.w3.org/copyright/software-license-2023/)
- **OASIS schemas**: OASIS IPR Policy and terms specified in schema files

See individual LICENSE files in each subdirectory for complete terms.

## Updating Schemas

When updating bundled schemas:

1. Verify the schema license permits redistribution
2. Download the official schema from the authoritative source
3. Update the schema file in the appropriate subdirectory
4. Update version information in this README
5. Verify the LICENSE file reflects current terms

## Attribution

These schemas are copyrighted by their respective organizations (W3C, OASIS) and are redistributed here under their respective licenses. All copyright notices in the schema files have been preserved.
