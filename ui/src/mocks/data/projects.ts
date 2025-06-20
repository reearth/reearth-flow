export type MockProject = {
  id: string;
  name: string;
  description: string;
  workspaceId: string;
  isArchived: boolean;
  isBasicAuthActive: boolean;
  basicAuthUsername: string;
  basicAuthPassword: string;
  sharedToken?: string;
  version: number;
  parameters: MockParameter[];
  createdAt: string;
  updatedAt: string;
};

export type MockParameter = {
  id: string;
  name: string;
  type: ParameterType;
  value: any;
  required: boolean;
  index: number;
  projectId: string;
  createdAt: string;
  updatedAt: string;
};

export type ParameterType = 
  | "CHOICE"
  | "COLOR" 
  | "DATETIME"
  | "FILE_FOLDER"
  | "MESSAGE"
  | "NUMBER"
  | "PASSWORD"
  | "TEXT"
  | "YES_NO"
  | "ATTRIBUTE_NAME"
  | "COORDINATE_SYSTEM"
  | "DATABASE_CONNECTION"
  | "GEOMETRY"
  | "REPROJECTION_FILE"
  | "WEB_CONNECTION";

export const mockProjects: MockProject[] = [
  {
    id: "project-1",
    name: "Data Processing Pipeline",
    description: "A comprehensive data processing pipeline for geospatial data",
    workspaceId: "workspace-1",
    isArchived: false,
    isBasicAuthActive: false,
    basicAuthUsername: "",
    basicAuthPassword: "",
    sharedToken: "shared-token-1",
    version: 1,
    parameters: [
      {
        id: "param-1",
        name: "input_format",
        type: "TEXT",
        value: "geojson",
        required: true,
        index: 0,
        projectId: "project-1",
        createdAt: "2024-01-01T10:00:00Z",
        updatedAt: "2024-01-01T10:00:00Z",
      },
      {
        id: "param-2",
        name: "output_crs",
        type: "COORDINATE_SYSTEM",
        value: "EPSG:4326",
        required: true,
        index: 1,
        projectId: "project-1",
        createdAt: "2024-01-01T10:00:00Z",
        updatedAt: "2024-01-01T10:00:00Z",
      },
    ],
    createdAt: "2024-01-01T10:00:00Z",
    updatedAt: "2024-01-15T10:00:00Z",
  },
  {
    id: "project-2",
    name: "Real-time Analytics",
    description: "Real-time data analytics and visualization workflows",
    workspaceId: "workspace-2",
    isArchived: false,
    isBasicAuthActive: true,
    basicAuthUsername: "analytics",
    basicAuthPassword: "secure123",
    version: 2,
    parameters: [
      {
        id: "param-3",
        name: "refresh_interval",
        type: "NUMBER",
        value: 30,
        required: false,
        index: 0,
        projectId: "project-2",
        createdAt: "2024-01-05T14:30:00Z",
        updatedAt: "2024-01-05T14:30:00Z",
      },
      {
        id: "param-4",
        name: "enable_alerts",
        type: "YES_NO",
        value: true,
        required: false,
        index: 1,
        projectId: "project-2",
        createdAt: "2024-01-05T14:30:00Z",
        updatedAt: "2024-01-05T14:30:00Z",
      },
    ],
    createdAt: "2024-01-05T14:30:00Z",
    updatedAt: "2024-01-20T09:15:00Z",
  },
  {
    id: "project-3",
    name: "Machine Learning Workflow",
    description: "Automated machine learning pipeline for prediction models",
    workspaceId: "workspace-2",
    isArchived: false,
    isBasicAuthActive: false,
    basicAuthUsername: "",
    basicAuthPassword: "",
    sharedToken: "shared-token-3",
    version: 1,
    parameters: [
      {
        id: "param-5",
        name: "model_type",
        type: "CHOICE",
        value: "random_forest",
        required: true,
        index: 0,
        projectId: "project-3",
        createdAt: "2024-01-10T16:45:00Z",
        updatedAt: "2024-01-10T16:45:00Z",
      },
      {
        id: "param-6",
        name: "training_split",
        type: "NUMBER",
        value: 0.8,
        required: true,
        index: 1,
        projectId: "project-3",
        createdAt: "2024-01-10T16:45:00Z",
        updatedAt: "2024-01-10T16:45:00Z",
      },
    ],
    createdAt: "2024-01-10T16:45:00Z",
    updatedAt: "2024-01-25T11:20:00Z",
  },
  {
    id: "project-4",
    name: "Data Visualization Dashboard",
    description: "Interactive dashboard for business intelligence",
    workspaceId: "workspace-3",
    isArchived: false,
    isBasicAuthActive: false,
    basicAuthUsername: "",
    basicAuthPassword: "",
    version: 3,
    parameters: [
      {
        id: "param-7",
        name: "chart_type",
        type: "CHOICE",
        value: "bar",
        required: false,
        index: 0,
        projectId: "project-4",
        createdAt: "2024-01-12T09:00:00Z",
        updatedAt: "2024-01-12T09:00:00Z",
      },
      {
        id: "param-8",
        name: "auto_refresh",
        type: "YES_NO",
        value: false,
        required: false,
        index: 1,
        projectId: "project-4",
        createdAt: "2024-01-12T09:00:00Z",
        updatedAt: "2024-01-12T09:00:00Z",
      },
    ],
    createdAt: "2024-01-12T09:00:00Z",
    updatedAt: "2024-01-28T15:30:00Z",
  },
  {
    id: "project-5",
    name: "Legacy Data Migration",
    description: "Migration workflow for legacy data systems",
    workspaceId: "workspace-1",
    isArchived: true,
    isBasicAuthActive: false,
    basicAuthUsername: "",
    basicAuthPassword: "",
    version: 1,
    parameters: [
      {
        id: "param-9",
        name: "batch_size",
        type: "NUMBER",
        value: 1000,
        required: true,
        index: 0,
        projectId: "project-5",
        createdAt: "2023-12-01T08:00:00Z",
        updatedAt: "2023-12-01T08:00:00Z",
      },
    ],
    createdAt: "2023-12-01T08:00:00Z",
    updatedAt: "2023-12-15T17:00:00Z",
  },
  {
    id: "project-6",
    name: "Design System Components",
    description: "Automated component generation and documentation",
    workspaceId: "workspace-4",
    isArchived: false,
    isBasicAuthActive: false,
    basicAuthUsername: "",
    basicAuthPassword: "",
    sharedToken: "shared-token-6",
    version: 1,
    parameters: [
      {
        id: "param-10",
        name: "theme",
        type: "CHOICE",
        value: "modern",
        required: false,
        index: 0,
        projectId: "project-6",
        createdAt: "2024-01-08T12:00:00Z",
        updatedAt: "2024-01-08T12:00:00Z",
      },
      {
        id: "param-11",
        name: "generate_docs",
        type: "YES_NO",
        value: true,
        required: false,
        index: 1,
        projectId: "project-6",
        createdAt: "2024-01-08T12:00:00Z",
        updatedAt: "2024-01-08T12:00:00Z",
      },
    ],
    createdAt: "2024-01-08T12:00:00Z",
    updatedAt: "2024-01-22T14:45:00Z",
  },
];
