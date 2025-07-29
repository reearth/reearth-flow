enum MockCMSVisibility {
  PUBLIC = "PUBLIC",
  PRIVATE = "PRIVATE",
}

enum MockCMSSchemaFieldType {
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

type MockCMSSchemaField = {
  fieldId: string;
  name: string;
  type: MockCMSSchemaFieldType;
  key: string;
  description: string;
};

type MockCMSSchema = {
  schemaId: string;
  fields: MockCMSSchemaField[];
};

export type MockCMSProject = {
  id: string;
  name: string;
  alias: string;
  description?: string;
  license?: string;
  readme?: string;
  workspaceId: string;
  visibility: MockCMSVisibility;
  createdAt: string;
  updatedAt: string;
};

export type MockCMSModel = {
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

export type MockCMSItem = {
  id: string;
  fields: JSON;
  createdAt: string;
  updatedAt: string;
};

export const mockCmsProjects: MockCMSProject[] = [
  {
    id: "proj-001",
    name: "Urban Development",
    alias: "urban-dev",
    description: "A project focused on urban development planning",
    license: "MIT",
    readme:
      "# Urban Development\nThis project contains data about urban development plans.",
    workspaceId: "ws-001",
    visibility: MockCMSVisibility.PUBLIC,
    createdAt: "2023-01-15T08:30:00Z",
    updatedAt: "2023-04-22T14:15:00Z",
  },
  {
    id: "proj-002",
    name: "Environmental Monitoring",
    alias: "env-monitor",
    description: "Environmental data collection and analysis",
    workspaceId: "ws-001",
    visibility: MockCMSVisibility.PRIVATE,
    createdAt: "2023-02-10T09:45:00Z",
    updatedAt: "2023-05-18T11:20:00Z",
  },
];

export const mockCmsModels: MockCMSModel[] = [
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
          type: MockCMSSchemaFieldType.TEXT,
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
          type: MockCMSSchemaFieldType.SELECT,
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

export const mockCmsItems: MockCMSItem[] = [
  {
    id: "item-001",
    fields: JSON.parse('{"name": "City Tower", "height": 150, "floors": 42}'),
    createdAt: "2023-01-25T14:20:00Z",
    updatedAt: "2023-04-28T09:10:00Z",
  },
  {
    id: "item-002",
    fields: JSON.parse(
      '{"type": "air_quality", "location": "downtown", "active": true}',
    ),
    createdAt: "2023-02-18T15:40:00Z",
    updatedAt: "2023-05-22T10:25:00Z",
  },
];
