# CMS Integration in Reearth-Flow

This document describes how to use the CMS (Content Management System) integration in Reearth-Flow API.

## Overview

The CMS integration allows Reearth-Flow to access data from Reearth-CMS through the internal gRPC API. This enables workflows to:
- List CMS projects and models
- Retrieve items from CMS models
- Export model data as GeoJSON
- Access asset URLs from CMS items

## Configuration

To enable CMS integration, set the following environment variables:

```bash
# CMS gRPC endpoint
REEARTH_CMS_ENDPOINT=cms-grpc-server:50051

# M2M authentication token
REEARTH_CMS_TOKEN=your-m2m-token

# User ID for metadata (optional)
REEARTH_CMS_USER_ID=system-user-id
```

## GraphQL API

### Queries

#### Get CMS Project

```graphql
query GetCMSProject($projectId: ID!) {
  cmsProject(projectIdOrAlias: $projectId) {
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

#### List CMS Projects

```graphql
query ListCMSProjects($workspaceId: ID!, $publicOnly: Boolean) {
  cmsProjects(workspaceId: $workspaceId, publicOnly: $publicOnly) {
    id
    name
    alias
    visibility
  }
}
```

#### List CMS Models

```graphql
query ListCMSModels($projectId: ID!) {
  cmsModels(projectId: $projectId) {
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

#### List CMS Items

```graphql
query ListCMSItems($projectId: ID!, $modelId: ID!, $page: Int, $pageSize: Int) {
  cmsItems(projectId: $projectId, modelId: $modelId, page: $page, pageSize: $pageSize) {
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

#### Get Model Export URL

```graphql
query GetModelExportURL($projectId: ID!, $modelId: ID!) {
  cmsModelExportUrl(projectId: $projectId, modelId: $modelId)
}
```

## Example Usage

### 1. Accessing CMS Data in a Workflow

When creating a workflow that needs to access CMS data:

```javascript
// Example: Processing CMS items with asset URLs
const cmsItems = await graphql(`
  query {
    cmsItems(projectId: "project-id", modelId: "model-id") {
      items {
        id
        fields
      }
    }
  }
`);

// Process items with asset URLs
cmsItems.items.forEach(item => {
  const assetField = item.fields.assetField;
  if (assetField && assetField.url) {
    // Use the asset URL in your workflow
    processAsset(assetField.url);
  }
});
```

### 2. Exporting GeoJSON Data

```javascript
// Get the export URL
const exportUrl = await graphql(`
  query {
    cmsModelExportUrl(projectId: "project-id", modelId: "model-id")
  }
`);

// Use the URL to download GeoJSON data
const geoJsonData = await fetch(exportUrl);
```

## Architecture

The CMS integration uses the following architecture:

```
┌─────────────────┐     ┌──────────────────┐     ┌───────────────┐
│  Flow API       │────▶│  CMS gRPC Client │────▶│  CMS Internal │
│  (GraphQL)      │     │  (Gateway)       │     │  API          │
└─────────────────┘     └──────────────────┘     └───────────────┘
```

### Components

1. **Gateway Interface** (`internal/usecase/gateway/cms.go`)
   - Defines the CMS gateway interface for the usecase layer

2. **gRPC Client** (`internal/infrastructure/cms/grpc_client.go`)
   - Implements the CMS gateway interface
   - Handles gRPC communication with CMS
   - Manages authentication via metadata

3. **Usecase/Interactor** (`internal/usecase/interactor/cms.go`)
   - Implements business logic
   - Handles permission checks
   - Provides CMS operations to the GraphQL layer

4. **Domain Types** (`pkg/cms/cms.go`)
   - Defines domain models for CMS entities
   - Provides type safety across the application

## Security

- All CMS operations require authentication
- Permission checks are performed for workspace access
- M2M tokens are used for server-to-server authentication
- User context is preserved through the request chain

## Future Enhancements

1. **Caching**: Implement caching for frequently accessed CMS data
2. **Webhooks**: Support CMS webhooks for real-time data updates
3. **Batch Operations**: Add support for batch item retrieval
4. **Field Filtering**: Allow filtering items by specific field values
5. **Write Operations**: Add support for creating/updating CMS data (if needed) 