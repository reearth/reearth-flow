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

// Type-specific configuration interfaces
export type ChoiceConfig = {
  choices: string[];
  displayMode?: "dropdown" | "radio";
  allowMultiple?: boolean;
};

export type CoordinateConfig = {
  x: string;
  y: string;
  z?: string;
  coordinateSystem?: string;
};

export type ColorConfig = {
  format?: "hex" | "rgb" | "hsl";
  allowAlpha?: boolean;
};

export type DatabaseConnectionConfig = {
  host: string;
  port: number;
  username: string;
  database?: string;
  ssl?: boolean;
};

export type GeometryConfig = {
  geometryType?:
    | "Point"
    | "LineString"
    | "Polygon"
    | "MultiPoint"
    | "MultiLineString"
    | "MultiPolygon";
  coordinateSystem?: string;
  allowEmpty?: boolean;
};

export type NumberConfig = {
  min?: number;
  max?: number;
  // step?: number;
  // precision?: number;
  // unit?: string;
};

export type TextConfig = {
  minLength?: number;
  maxLength?: number;
  multiline?: boolean;
  // pattern?: string;
};

export type DateTimeConfig = {
  format?: string;
  timezone?: string;
  allowTime?: boolean;
  minDate?: string;
  maxDate?: string;
};

export type WebConnectionConfig = {
  allowedProtocols?: string[];
  requiresAuth?: boolean;
  timeout?: number;
};

export type FileConfig = {
  allowedExtensions?: string[];
  maxSize?: number;
  allowMultiple?: boolean;
  accept?: string;
};

// Conditional config type based on VarType
export type ProjectVariableConfig<T extends VarType> = T extends "choice"
  ? ChoiceConfig
  : T extends "coordinate_system"
    ? CoordinateConfig
    : T extends "color"
      ? ColorConfig
      : T extends "database_connection"
        ? DatabaseConnectionConfig
        : T extends "geometry"
          ? GeometryConfig
          : T extends "number"
            ? NumberConfig
            : T extends "text"
              ? TextConfig
              : T extends "datetime"
                ? DateTimeConfig
                : T extends "web_connection"
                  ? WebConnectionConfig
                  : T extends "file_folder"
                    ? FileConfig
                    : undefined;

export type ProjectVariable<T extends VarType = VarType> = {
  id: string;
  name: string;
  defaultValue: any;
  type: T;
  required: boolean;
  public: boolean;
  config?: ProjectVariableConfig<T>;
  createdAt?: string;
  updatedAt?: string;
  projectId?: string;
};

// Convenience type for when we don't know the specific type
export type AnyProjectVariable = ProjectVariable<VarType>;

export type CreateProjectVariable = {
  projectVariable?: AnyProjectVariable;
} & ApiResponse;

export type UpdateProjectVariable = {
  projectVariable?: AnyProjectVariable;
} & ApiResponse;
