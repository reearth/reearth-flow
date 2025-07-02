# CMS Integration Example

This document shows how to use the CMS integration in Reearth-Flow API.

## Environment Setup

Set the following environment variables to enable CMS integration:

```bash
export REEARTH_CMS_ENDPOINT=localhost:50051
export REEARTH_CMS_TOKEN=your-m2m-token
export REEARTH_CMS_USER_ID=system-user-id
```

## GraphQL Query Examples

### 1. Get a CMS Project

```graphql
query GetCMSProject {
  cmsProject(projectIdOrAlias: "my-project") {
    id
    name
    alias
    description
    workspaceId
    visibility
    createdAt
    updatedAt
  }
}
```

### 2. List CMS Models

```graphql
query ListCMSModels {
  cmsModels(projectId: "project-id") {
    id
    name
    description
    key
    schema {
      schemaId
      fields {
        fieldId
        name
        type
        key
        description
      }
    }
    publicApiEp
    editorUrl
  }
}
```

### 3. List CMS Items with Asset URLs

```graphql
query ListCMSItems {
  cmsItems(
    projectId: "project-id"
    modelId: "model-id"
    page: 1
    pageSize: 20
  ) {
    items {
      id
      fields
      createdAt
      updatedAt
    }
    totalCount
  }
}
```

### 4. Get Model Export URL

```graphql
query GetModelExportURL {
  cmsModelExportUrl(
    projectId: "project-id"
    modelId: "model-id"
  )
}
```

## Accessing Asset URLs in Items

When retrieving items from CMS, asset fields will contain URLs that can be used directly in workflows:

```json
{
  "data": {
    "cmsItems": {
      "items": [
        {
          "id": "item-1",
          "fields": {
            "title": "My Item",
            "image": {
              "id": "asset-123",
              "url": "https://cms.example.com/assets/asset-123.jpg",
              "name": "photo.jpg",
              "size": 1024000
            }
          }
        }
      ]
    }
  }
}
```

## Integration with Workflows

The CMS data can be used in workflows through the GraphQL API. For example:

1. **Data Source**: Use CMS items as input data for workflows
2. **Asset Processing**: Access asset URLs from CMS items for processing
3. **GeoJSON Export**: Use the export URL to get GeoJSON data for geospatial workflows

## Complete Integration Flow

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌────────────┐
│   Client    │────▶│  Flow API   │────▶│ CMS gRPC    │────▶│    CMS     │
│  (GraphQL)  │     │  (GraphQL)  │     │   Client    │     │  Internal  │
└─────────────┘     └─────────────┘     └─────────────┘     │    API     │
                                                              └────────────┘
```

## Notes

- All CMS operations require authentication
- Permission checks are performed based on workspace access
- The gRPC connection uses M2M tokens for server-to-server authentication
- Asset URLs returned from CMS can be used directly in workflows 