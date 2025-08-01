import {
  CmsModelFragment,
  CmsProjectFragment,
  CmsItemFragment,
  CmsVisibility as GraphqlCmsVisibility,
  CmsSchemaFieldType as GraphQlCmsSchemaFieldType,
} from "@flow/lib/gql/__gen__/graphql";

export const mockCmsProjects: CmsProjectFragment[] = [
  {
    id: "proj-001",
    name: "Urban Development",
    alias: "urban-dev",
    description: "A project focused on urban development planning",
    license: "MIT",
    readme:
      "# Urban Development\nThis project contains data about urban development plans.",
    workspaceId: "workspace-1",
    visibility: GraphqlCmsVisibility.Public,
    createdAt: "2023-01-15T08:30:00Z",
    updatedAt: "2023-04-22T14:15:00Z",
  },
  {
    id: "proj-002",
    name: "Environmental Monitoring",
    alias: "env-monitor",
    description: "Environmental data collection and analysis",
    workspaceId: "workspace-1",
    visibility: GraphqlCmsVisibility.Private,
    createdAt: "2023-02-10T09:45:00Z",
    updatedAt: "2023-05-18T11:20:00Z",
  },
];

export const mockCmsModels: CmsModelFragment[] = [
  {
    id: "model-001",
    projectId: "proj-001",
    name: "Building",
    description: "Building information model",
    key: "building",
    schema: {
      schemaId: "schema-001",
      fields: [
        {
          fieldId: "field-001",
          name: "Building Name",
          type: GraphQlCmsSchemaFieldType.Text,
          key: "name",
          description: "Name of the building",
        },
      ],
    },
    publicApiEp: "/api/buildings",
    editorUrl: "/cms/editor/buildings",
    createdAt: "2023-01-20T10:00:00Z",
    updatedAt: "2023-04-25T16:30:00Z",
  },
  {
    id: "model-002",
    projectId: "proj-002",
    name: "Sensor",
    description: "Environmental sensor data",
    key: "sensor",
    schema: {
      schemaId: "schema-002",
      fields: [
        {
          fieldId: "field-002",
          name: "Sensor Type",
          type: GraphQlCmsSchemaFieldType.Select,
          key: "type",
          description: "Type of environmental sensor",
        },
      ],
    },
    publicApiEp: "/api/sensors",
    editorUrl: "/cms/editor/sensors",
    createdAt: "2023-02-15T11:30:00Z",
    updatedAt: "2023-05-20T13:45:00Z",
  },
];

export const mockCmsItems: (CmsItemFragment & {
  projectId: string;
  modelId: string;
})[] = [
  {
    id: "item-001",
    projectId: "proj-001",
    modelId: "model-001",
    fields: {
      name: "City Tower",
      height: 150,
      floors: 42,
    },
    createdAt: "2023-01-25T14:20:00Z",
    updatedAt: "2023-04-28T09:10:00Z",
  },
  {
    id: "item-002",
    projectId: "proj-002",
    modelId: "model-002",
    fields: {
      type: "air_quality",
      location: "downtown",
      active: true,
    },
    createdAt: "2023-02-18T15:40:00Z",
    updatedAt: "2023-05-22T10:25:00Z",
  },
  {
    id: "item-003",
    projectId: "proj-001",
    modelId: "model-001",
    fields: {
      name: "Metro Plaza",
      height: 85,
      floors: 25,
    },
    createdAt: "2023-01-30T11:15:00Z",
    updatedAt: "2023-04-30T14:20:00Z",
  },
];
