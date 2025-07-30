export type CmsVisibility = "public" | "private";

export type CmsSchemaFieldType =
  | "text"
  | "text_area"
  | "rich_text"
  | "mark_down_text"
  | "asset"
  | "date"
  | "bool"
  | "select"
  | "tag"
  | "integer"
  | "number"
  | "reference"
  | "checkbox"
  | "url"
  | "group"
  | "geometry_object"
  | "geometry_editor";

export type CmsSchemaField = {
  fieldId: string;
  name: string;
  type: CmsSchemaFieldType;
  key: string;
  description?: string;
};

export type CmsSchema = {
  schemaId: string;
  fields: CmsSchemaField[];
};

export type CmsProject = {
  id: string;
  name: string;
  alias: string;
  description?: string | null;
  license?: string | null;
  readme?: string | null;
  workspaceId: string;
  visibility: CmsVisibility;
  createdAt: string;
  updatedAt: string;
};

export type CmsModel = {
  id: string;
  projectId: string;
  name: string;
  description: string;
  key: string;
  schema: CmsSchema;
  publicApiEp: string;
  editorUrl: string;
  createdAt: string;
  updatedAt: string;
};

export type CmsItem = {
  id: string;
  fields: Record<string, any>;
  createdAt: string;
  updatedAt: string;
};
