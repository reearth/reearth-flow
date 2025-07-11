# XML Schema Validation Module

This module provides XML validation functionality with support for remote schema resolution and caching.

## Why SchemaResolver is Required

### libxml2 Constraints

The XMLValidator uses libxml2 for schema validation, which has the following constraints:

1. **Local File Access Only**: libxml2 can only read schema files from the local filesystem. It cannot directly fetch schemas from HTTP/HTTPS URLs.

2. **No Built-in URL Resolution**: When a schema imports or includes other schemas via HTTP/HTTPS URLs, libxml2 cannot resolve these dependencies automatically.

3. **Path-based Resolution**: libxml2 resolves relative schema imports based on filesystem paths, not URLs.

### The SchemaResolver Solution

To overcome these limitations, we implement a SchemaResolver that:

1. **Pre-fetches Remote Schemas**: Downloads all schemas from HTTP/HTTPS URLs before validation
2. **Resolves Dependencies**: Recursively fetches all imported/included schemas
3. **Rewrites Schema Locations**: Updates schema import/include paths to point to local cached files
4. **Caches Schemas Locally**: Stores schemas on the filesystem where libxml2 can access them

## How Schema Resolution Works

### 1. Dependency Discovery

The resolver parses XML schemas to find dependencies through:
- `<xs:import>` elements (for schemas with different namespaces)
- `<xs:include>` elements (for schemas with the same namespace)

```xml
<!-- Example schema with dependencies -->
<xs:schema xmlns:xs="http://www.w3.org/2001/XMLSchema">
    <xs:import namespace="http://example.com/common" 
               schemaLocation="https://example.com/schemas/common.xsd"/>
    <xs:include schemaLocation="https://example.com/schemas/types.xsd"/>
</xs:schema>
```

### 2. Recursive Resolution

The resolution process:

1. **Start with Root Schema**: Fetch the main schema from its URL
2. **Parse for Dependencies**: Extract all import/include URLs
3. **Fetch Dependencies**: Download each referenced schema
4. **Repeat Recursively**: Process each dependency for its own dependencies
5. **Handle Circular References**: Track visited schemas to avoid infinite loops

### 3. Schema Rewriting

Before saving schemas locally, the resolver rewrites schema locations:

```xml
<!-- Original -->
<xs:import schemaLocation="https://example.com/schemas/common.xsd"/>

<!-- Rewritten -->
<xs:import schemaLocation="file:///tmp/cache/a1b2c3d4-common.xsd"/>
```

### 4. Local Cache Structure

Schemas are cached with hashed filenames to avoid conflicts:
```
/tmp/reearth-flow-xmlvalidator-schema/
├── 12345678-root.xsd       # Root schema
├── 87654321-common.xsd     # Imported schema
└── abcdef01-types.xsd      # Included schema
```

## Why Schema Dependencies Work

### Handling Complex Dependencies

The resolver can handle schemas that depend on other schemas because:

1. **Topological Resolution**: Schemas are fetched in dependency order
2. **Complete Graph**: All transitive dependencies are resolved
3. **Circular Reference Detection**: Prevents infinite loops in circular dependencies
4. **Namespace Preservation**: Maintains namespace relationships during rewriting

### Example: Multi-level Dependencies

```
root.xsd
├── imports common.xsd
│   └── imports types.xsd
│       └── imports primitives.xsd
└── includes extensions.xsd
    └── imports common.xsd (already resolved)
```

The resolver:
1. Uses breadth-first traversal to discover all dependencies
2. Tracks visited schemas to handle circular references
3. Fetches each unique schema only once
4. Stores all schemas with their direct dependencies
5. Rewrites all cross-references to use local paths
6. Ensures libxml2 can resolve all imports/includes locally

## Cache Strategy

### Filesystem-based Caching

**Filesystem Cache**: Persists schemas locally for libxml2 access
- Schemas are fetched once and stored on disk
- libxml2 can directly read from cached files
- Cache persists across workflow executions

### Cache Key Generation

The cache key generation uses HashMap's `DefaultHasher` for consistent hashing:

```rust
// Generate a cache key for a schema URL
fn generate_cache_key(url: &str) -> String {
    let hash = {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        url.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    };
    let filename = url.split('/').last().unwrap_or("schema.xsd");
    format!("xmlvalidator-schema/{}-{}", &hash[..8], filename)
}
```

This approach:
- Uses the standard library's `DefaultHasher` from HashMap
- Takes the first 8 characters of the hex-encoded hash for brevity
- Appends the original filename for human readability
- Prefixes with "xmlvalidator-schema/" for organization

### Cache Checking

Before fetching schemas:
1. Check if root schema is cached
2. Verify all dependencies are also cached
3. Only fetch if any schema is missing

## Benefits

1. **Network Efficiency**: Schemas are fetched once and reused
2. **Offline Capability**: Cached schemas work without network access
3. **Performance**: Local file access is faster than HTTP requests
4. **Reliability**: Reduces dependency on external schema availability