import { VarType } from "@flow/types";

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
      // For a choice, return empty string as default
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
      // Return the current date and time by default
      return new Date();

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
      return "some text";

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
