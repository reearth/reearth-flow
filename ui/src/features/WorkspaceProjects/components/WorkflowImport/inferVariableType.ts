import { VarType } from "@flow/types";

export function inferProjectVariableType(value: any, name: string): VarType {
  const normalizedName = name.toLowerCase();

  // Handle null values by inferring from name patterns
  if (value === null || value === undefined) {
    if (
      normalizedName.includes("path") ||
      normalizedName.includes("file") ||
      normalizedName.includes("folder")
    ) {
      return "file_folder";
    }
    if (normalizedName.includes("output") || normalizedName.includes("dir")) {
      return "file_folder";
    }
    if (
      normalizedName.includes("password") ||
      normalizedName.includes("secret")
    ) {
      return "password";
    }
    if (normalizedName.includes("date") || normalizedName.includes("time")) {
      return "datetime";
    }
    if (normalizedName.includes("color")) {
      return "color";
    }
    if (
      normalizedName.includes("connection") ||
      normalizedName.includes("url") ||
      normalizedName.includes("endpoint")
    ) {
      return "web_connection";
    }
    // Default fallback for null values
    return "text";
  }

  // Infer from actual value types
  if (typeof value === "number") {
    return "number";
  }

  if (typeof value === "boolean") {
    return "yes_no";
  }

  if (Array.isArray(value)) {
    // Arrays should be mapped to array type
    return "array";
  }

  if (typeof value === "string") {
    // String pattern matching
    if (
      normalizedName.includes("path") ||
      normalizedName.includes("file") ||
      normalizedName.includes("folder")
    ) {
      return "file_folder";
    }
    if (
      normalizedName.includes("password") ||
      normalizedName.includes("secret")
    ) {
      return "password";
    }
    if (
      normalizedName.includes("color") &&
      (value.startsWith("#") || value.startsWith("rgb"))
    ) {
      return "color";
    }
    if (normalizedName.includes("date") || normalizedName.includes("time")) {
      return "datetime";
    }
    if (
      normalizedName.includes("connection") ||
      value.startsWith("http") ||
      value.startsWith("ws")
    ) {
      return "web_connection";
    }
  }

  // Default fallback
  return "text";
}

export function getDefaultValue(value: any, type: VarType): any {
  // If value is null/undefined, provide appropriate defaults
  if (value === null || value === undefined) {
    switch (type) {
      case "array":
        return [];
      case "number":
        return 0;
      case "yes_no":
        return false;
      case "choice":
        return "";
      case "text":
      case "file_folder":
      case "password":
      case "datetime":
      case "color":
      case "web_connection":
      default:
        return "";
    }
  }

  // For arrays, preserve as-is if type is array, otherwise convert based on target type
  if (Array.isArray(value)) {
    if (type === "array") {
      return value; // Keep arrays as arrays for array type
    } else if (type === "choice") {
      return value.join(", "); // Convert to comma-separated for choice type
    } else if (type === "text") {
      return JSON.stringify(value); // Convert to JSON string for text type
    }
  }

  // For non-null values, use as-is
  return value;
}
