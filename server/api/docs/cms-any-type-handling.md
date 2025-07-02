# CMS Any Type Handling

This document explains how the `convertAnyToInterface` function handles protobuf `Any` types in the CMS integration.

## Overview

The `google.protobuf.Any` type is used by CMS to store field values of different types in a single map. The `convertAnyToInterface` function unpacks these `Any` values into their corresponding Go types.

## Supported Types

The function supports the following protobuf well-known types:

### Basic Types
- `google.protobuf.StringValue` → `string`
- `google.protobuf.Int32Value` → `int32`
- `google.protobuf.Int64Value` → `int64`
- `google.protobuf.DoubleValue` → `float64`
- `google.protobuf.FloatValue` → `float32`
- `google.protobuf.BoolValue` → `bool`

### Complex Types
- `google.protobuf.Timestamp` → `time.Time`
- `google.protobuf.Struct` → `map[string]interface{}`
- `google.protobuf.Value` → `interface{}` (dynamic type)
- `google.protobuf.ListValue` → `[]interface{}`

## Implementation Details

The function uses a type switch on the `TypeUrl` field of the `Any` message to determine the actual type, then unmarshals the `Value` field accordingly:

```go
switch a.TypeUrl {
case "type.googleapis.com/google.protobuf.StringValue":
    var sv wrapperspb.StringValue
    if err := protobuf.Unmarshal(a.Value, &sv); err == nil {
        return sv.Value
    }
// ... other cases
}
```

## Example Usage

When retrieving CMS items, the fields are stored as `map[string]*anypb.Any`. The function converts these to usable Go values:

```json
// Input (protobuf Any)
{
  "title": {
    "typeUrl": "type.googleapis.com/google.protobuf.StringValue",
    "value": <binary data>
  },
  "count": {
    "typeUrl": "type.googleapis.com/google.protobuf.Int32Value",
    "value": <binary data>
  }
}

// Output (Go map)
{
  "title": "My Item Title",
  "count": 42
}
```

## Asset Field Handling

For asset fields, CMS typically returns a `Struct` containing:
- `id`: Asset ID
- `url`: Asset URL
- `name`: File name
- `size`: File size in bytes

The function automatically converts these to `map[string]interface{}`:

```go
{
  "image": map[string]interface{}{
    "id": "asset-123",
    "url": "https://cms.example.com/assets/asset-123.jpg",
    "name": "photo.jpg",
    "size": float64(1024000),
  }
}
```

## Error Handling

If the function cannot unmarshal an `Any` value:
1. It attempts generic protobuf message unmarshaling
2. If that fails, it logs a warning and returns the raw byte value
3. This ensures the function never panics and always returns something

## Testing

The function is thoroughly tested with various protobuf types. See `grpc_client_test.go` for test examples. 