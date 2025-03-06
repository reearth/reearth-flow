# Thrift Definitions for reearth-flow

This directory contains Thrift definition files for the reearth-flow API services.

## Document Service

The `document.thrift` file defines the Document service, which provides methods for managing documents:

- `GetLatest`: Retrieves the latest version of a document
- `GetHistory`: Retrieves the version history of a document
- `Rollback`: Rolls back a document to a specific version

## Generating Code

To generate code from the Thrift definitions, use the provided script:

```bash
# From the api directory
./scripts/generate_thrift.sh
```

This will generate Go code in the `proto` directory.

## Requirements

- Apache Thrift compiler (version 0.16.0 or later)

To install the Thrift compiler:

### macOS
```bash
brew install thrift
```

### Ubuntu/Debian
```bash
apt-get install thrift-compiler
```

## Manual Generation

If you need to manually generate code, you can use the following command:

```bash
thrift -r --gen go:package_prefix=github.com/reearth/reearth-flow/api/ -out /path/to/api /path/to/api/thrift/document.thrift
``` 