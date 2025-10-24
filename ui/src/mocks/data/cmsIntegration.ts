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
    name: "Environmental Monitoring",
    alias: "env-monitor",
    description: "Environmental data collection and analysis",
    workspaceId: "workspace-1",
    topics: ["environment", "sensors", "data"],
    starCount: 42,
    visibility: GraphqlCmsVisibility.Public,
    createdAt: "2023-02-10T09:45:00Z",
    updatedAt: "2023-05-18T11:20:00Z",
  },
];

export const mockCmsModels: CmsModelFragment[] = [
  {
    id: "model-001",
    projectId: "proj-001",
    name: "Sensor",
    description: "Environmental sensor data",
    key: "sensor",
    schema: {
      schemaId: "schema-001",
      fields: [
        {
          fieldId: "field-001",
          name: "Name",
          type: GraphQlCmsSchemaFieldType.Text,
          key: "name",
          description: "Name of the building or area",
        },
        {
          fieldId: "field-002",
          name: "Location",
          type: GraphQlCmsSchemaFieldType.Select,
          key: "location",
          description: "Geographic location",
        },
        {
          fieldId: "field-003",
          name: "Data Url",
          type: GraphQlCmsSchemaFieldType.Url,
          key: "data_url",
          description: "URL for the data source",
        },
        {
          fieldId: "field-004",
          name: "City GML",
          type: GraphQlCmsSchemaFieldType.Asset,
          key: "cityGml",
          description: "City GML file for the area",
        },
      ],
    },
    publicApiEp: "/api/global-sensors",
    editorUrl: "/cms/editor/global-sensors",
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
      name: "Thames Estuary Monitor",
      location: "London, UK",
      data_url: "https://data.london.gov.uk/sensors/thames-estuary",
      cityGml: "https://assets.london.gov.uk/citygml/estuary-zone.gml",
    },
    createdAt: "2023-03-01T09:30:00Z",
    updatedAt: "2023-05-25T16:45:00Z",
  },
  {
    id: "item-002",
    projectId: "proj-001",
    modelId: "model-001",
    fields: {
      name: "Hudson Bay Sensor",
      location: "New York, USA",
      data_url: "https://data.nyc.gov/environmental/hudson-bay",
      cityGml: "https://assets.nyc.gov/citygml/hudson-district.gml",
    },
    createdAt: "2023-03-05T12:15:00Z",
    updatedAt: "2023-05-28T08:20:00Z",
  },
  {
    id: "item-003",
    projectId: "proj-001",
    modelId: "model-001",
    fields: {
      name: "Sydney Harbor Monitor",
      location: "Sydney, Australia",
      data_url: "https://data.sydney.gov.au/sensors/harbor-bridge",
      cityGml: "https://assets.sydney.gov.au/citygml/harbor-zone.gml",
    },
    createdAt: "2023-03-10T14:40:00Z",
    updatedAt: "2023-06-02T11:10:00Z",
  },
  {
    id: "item-004",
    projectId: "proj-001",
    modelId: "model-001",
    fields: {
      name: "Seine River Station",
      location: "Paris, France",
      data_url: "https://data.paris.fr/environmental/seine-central",
      cityGml: "https://assets.paris.fr/citygml/seine-area.gml",
    },
    createdAt: "2023-03-15T10:25:00Z",
    updatedAt: "2023-06-05T13:30:00Z",
  },
  {
    id: "item-005",
    projectId: "proj-001",
    modelId: "model-001",
    fields: {
      name: "Berlin Urban Sensor",
      location: "Berlin, Germany",
      data_url: "https://data.berlin.de/sensors/mitte-district",
      cityGml: "https://assets.berlin.de/citygml/mitte-zone.gml",
    },
    createdAt: "2023-03-20T16:50:00Z",
    updatedAt: "2023-06-08T09:15:00Z",
  },
  {
    id: "item-006",
    projectId: "proj-001",
    modelId: "model-001",
    fields: {
      name: "Toronto Waterfront Monitor",
      location: "Toronto, Canada",
      data_url: "https://data.toronto.ca/sensors/waterfront",
      cityGml: "https://assets.toronto.ca/citygml/waterfront-zone.gml",
    },
    createdAt: "2023-03-25T11:35:00Z",
    updatedAt: "2023-06-12T14:20:00Z",
  },
  {
    id: "item-007",
    projectId: "proj-001",
    modelId: "model-001",
    fields: {
      name: "São Paulo Urban Station",
      location: "São Paulo, Brazil",
      data_url: "https://data.sp.gov.br/sensors/centro-historico",
      cityGml: "https://assets.sp.gov.br/citygml/centro-area.gml",
    },
    createdAt: "2023-04-01T08:45:00Z",
    updatedAt: "2023-06-15T10:30:00Z",
  },
  {
    id: "item-008",
    projectId: "proj-001",
    modelId: "model-001",
    fields: {
      name: "Cape Town Harbor Sensor",
      location: "Cape Town, South Africa",
      data_url: "https://data.capetown.gov.za/sensors/v-and-a-waterfront",
      cityGml: "https://assets.capetown.gov.za/citygml/harbor-zone.gml",
    },
    createdAt: "2023-04-05T13:20:00Z",
    updatedAt: "2023-06-18T16:45:00Z",
  },
  {
    id: "item-009",
    projectId: "proj-001",
    modelId: "model-001",
    fields: {
      name: "Mumbai Coastal Monitor",
      location: "Mumbai, India",
      data_url: "https://data.mumbai.gov.in/sensors/marine-drive",
      cityGml: "https://assets.mumbai.gov.in/citygml/coastal-zone.gml",
    },
    createdAt: "2023-04-10T15:10:00Z",
    updatedAt: "2023-06-22T12:25:00Z",
  },
  {
    id: "item-010",
    projectId: "proj-001",
    modelId: "model-001",
    fields: {
      name: "Singapore Marina Sensor",
      location: "Singapore",
      data_url: "https://data.gov.sg/sensors/marina-bay",
    },
    createdAt: "2023-04-15T09:55:00Z",
    updatedAt: "2023-06-25T11:40:00Z",
  },
];
