import { VarType, ProjectVariableConfig } from "@flow/types";

// type DatabaseConnection = {
//   host: string;
//   port: number;
//   username: string;
//   password: string;
// };

// type Geometry = {
//   type: string;
//   coordinates: number[];
// };

export function getDefaultValueForProjectVar(type: VarType): any {
  switch (type) {
    case "attribute_name":
      // Default should be empty for user input
      return "";

    case "choice":
      // For a choice, return the default selected value (empty string for no selection)
      return "";

    case "color":
      // Default color should be empty for user selection
      return "";

    case "coordinate_system":
      // Default coordinate system might be an empty string or a standard EPSG code.
      return "";

    // case "database_connection":
    //   // An example default connection object
    //   const defaultConnection: DatabaseConnection = {
    //     host: "",
    //     port: 0,
    //     username: "",
    //     password: "",
    //   };
    //   return defaultConnection;

    case "datetime":
      // Return ISO string for datetime
      return new Date().toISOString();

    case "file_folder":
      // Default file/folder path can be an empty string or a predefined path if needed.
      return "";

    // case "geometry":
    //   // A basic geometry object, here represented as a Point at coordinates [0, 0]
    //   const defaultGeometry: Geometry = {
    //     type: "Point",
    //     coordinates: [0, 0],
    //   };
    //   return defaultGeometry;

    case "message":
      // Default message as an empty string
      return "";

    case "number":
      // Default number value, typically 0
      return 0;

    case "password":
      // Default for a password field is typically an empty string (never pre-populate real passwords)
      return "";

    case "reprojection_file":
      // Assuming a file path or identifier, empty string by default
      return "";

    case "text":
      // Default text as an empty string
      return "";

    case "web_connection":
      // Web connection might include URL and other details; default here is an empty string or object.
      return "";

    case "yes_no":
      // Represent a yes/no value as a boolean; defaulting to false.
      return false;

    case "unsupported":
      // For unsupported types, you might return undefined or null
      return undefined;

    default:
      // Fallback for any future cases or errors
      return null;
  }
}

/**
 * Get default configuration for a project variable type
 */
export function getDefaultConfigForProjectVar<T extends VarType>(
  type: T,
): ProjectVariableConfig<T> {
  switch (type) {
    case "choice":
      return {
        choices: ["Option 1", "Option 2", "Option 3"],
        displayMode: "dropdown",
        allowMultiple: false,
      } as ProjectVariableConfig<T>;

    case "coordinate_system":
      return {
        x: "x",
        y: "y",
        z: undefined,
        coordinateSystem: "EPSG:4326",
      } as ProjectVariableConfig<T>;

    case "color":
      return {
        format: "hex",
        allowAlpha: false,
      } as ProjectVariableConfig<T>;

    case "database_connection":
      return {
        host: "",
        port: 5432,
        username: "",
        database: "",
        ssl: false,
      } as ProjectVariableConfig<T>;

    case "geometry":
      return {
        geometryType: "Point",
        coordinateSystem: "EPSG:4326",
        allowEmpty: false,
      } as ProjectVariableConfig<T>;

    case "number":
      return {
        min: undefined,
        max: undefined,
        step: 1,
        precision: undefined,
        unit: undefined,
      } as ProjectVariableConfig<T>;

    case "text":
      return {
        minLength: undefined,
        maxLength: undefined,
        pattern: undefined,
        multiline: false,
      } as ProjectVariableConfig<T>;

    case "datetime":
      return {
        format: "YYYY-MM-DD HH:mm:ss",
        timezone: undefined,
        allowTime: true,
        minDate: undefined,
        maxDate: undefined,
      } as ProjectVariableConfig<T>;

    case "web_connection":
      return {
        allowedProtocols: ["http", "https"],
        requiresAuth: false,
        timeout: 30000,
      } as ProjectVariableConfig<T>;

    case "file_folder":
      return {
        allowedExtensions: undefined,
        maxSize: undefined,
        allowMultiple: false,
        accept: undefined,
      } as ProjectVariableConfig<T>;

    // Types that don't have config
    case "attribute_name":
    case "message":
    case "password":
    case "reprojection_file":
    case "yes_no":
    case "unsupported":
    default:
      return undefined as ProjectVariableConfig<T>;
  }
}
