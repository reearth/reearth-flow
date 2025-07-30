enum MockCmsVisibility {
  PUBLIC = "PUBLIC",
  PRIVATE = "PRIVATE",
}

enum MockCmsSchemaFieldType {
  TEXT = "TEXT",
  TEXTAREA = "TEXTAREA",
  RICHTEXT = "RICHTEXT",
  MARKDOWNTEXT = "MARKDOWNTEXT",
  ASSET = "ASSET",
  DATE = "DATE",
  BOOL = "BOOL",
  SELECT = "SELECT",
  TAG = "TAG",
  INTEGER = "INTEGER",
  NUMBER = "NUMBER",
  REFERENCE = "REFERENCE",
  CHECKBOX = "CHECKBOX",
  URL = "URL",
  GROUP = "GROUP",
  GEOMETRYOBJECT = "GEOMETRYOBJECT",
  GEOMETRYEDITOR = "GEOMETRYEDITOR",
}

type MockCmsSchemaField = {
  fieldId: string;
  name: string;
  type: MockCmsSchemaFieldType;
  key: string;
  description: string;
};

type MockCMSSchema = {
  schemaId: string;
  fields: MockCmsSchemaField[];
};

export type MockCmsProject = {
  id: string;
  name: string;
  alias: string;
  description?: string;
  license?: string;
  readme?: string;
  workspaceId: string;
  visibility: MockCmsVisibility;
  createdAt: string;
  updatedAt: string;
};

export type MockCmsModel = {
  id: string;
  projectId: string;
  name: string;
  description: string;
  key: string;
  schema: MockCMSSchema;
  publicApiEp: string;
  editorUrl: string;
  createdAt: string;
  updatedAt: string;
};

export type MockCmsItem = {
  id: string;
  fields: Record<string, any>;
  createdAt: string;
  updatedAt: string;
};

export const mockCmsProjects: MockCmsProject[] = [
  {
    id: "proj-001",
    name: "Urban Development",
    alias: "urban-dev",
    description: "A project focused on urban development planning",
    license: "MIT",
    readme:
      "# Urban Development\nThis project contains data about urban development plans.",
    workspaceId: "ws-001",
    visibility: MockCmsVisibility.PUBLIC,
    createdAt: "2023-01-15T08:30:00Z",
    updatedAt: "2023-04-22T14:15:00Z",
  },
  {
    id: "proj-002",
    name: "Environmental Monitoring",
    alias: "env-monitor",
    description: "Environmental data collection and analysis",
    workspaceId: "ws-001",
    visibility: MockCmsVisibility.PRIVATE,
    createdAt: "2023-02-10T09:45:00Z",
    updatedAt: "2023-05-18T11:20:00Z",
  },
];

export const mockCmsModels: MockCmsModel[] = [
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
          type: MockCmsSchemaFieldType.TEXT,
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
          type: MockCmsSchemaFieldType.SELECT,
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

export const mockCmsItems: MockCmsItem[] = [
  {
    id: "item-001",
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
    fields: {
      type: "air_quality",
      location: "downtown",
      active: true,
    },
    createdAt: "2023-02-18T15:40:00Z",
    updatedAt: "2023-05-22T10:25:00Z",
  },
];
