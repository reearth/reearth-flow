import { ApiResponse } from "./api";

export type VarType =
  | "attribute_name"
  | "choice"
  | "color"
  | "coordinate_system"
  | "database_connection"
  | "datetime"
  | "file_folder"
  | "geometry"
  | "message"
  | "number"
  | "password"
  | "reprojection_file"
  | "text"
  | "web_connection"
  | "yes_no"
  | "unsupported";

export type UserParameter = {
  id: string;
  name: string;
  value: any;
  type: VarType; // TODO: use ParameterType
  required: boolean;
  createdAt?: string;
  updatedAt?: string;
  projectId?: string;
};

export type CreateUserParamater = {
  userParameter?: UserParameter;
} & ApiResponse;
