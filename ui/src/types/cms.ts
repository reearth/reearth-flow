import { ApiResponse } from "./api";

enum CMSVisibility {
  PUBLIC = "PUBLIC",
  PRIVATE = "PRIVATE",
}

enum CMSSchemaFieldType {
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

type CMSSchemaField = {
  fieldId: string;
  name: string;
  type: CMSSchemaFieldType;
  key: string;
  description: string;
};

type CMSSchema = {
  schemaId: string;
  fields: CMSSchemaField;
};

export type CMSProject = {
  id: string;
  name: string;
  alias: string;
  description?: string;
  license?: string;
  readme?: string;
  workspaceId: string;
  visibility: CMSVisibility;
  createdAt: string;
  updatedAt: string;
};

export type CMSModel = {
  id: string;
  projectId: string;
  name: string;
  description: string;
  key: string;
  schema: CMSSchema;
  publicApiEp: string;
  editorUrl: string;
  createdAt: string;
  updatedAt: string;
};

export type CMSItem = {
  id: string;
  fields: JSON;
  createdAt: string;
  updatedAt: string;
};
export type GetCMSProject = {
  cmsProjects?: CMSProject;
} & ApiResponse;

export type GetCMSProjects = {
  cmsProjects?: CMSProject[];
} & ApiResponse;

export type GetCMSModelExportUrl = {
  url: string;
} & ApiResponse;
