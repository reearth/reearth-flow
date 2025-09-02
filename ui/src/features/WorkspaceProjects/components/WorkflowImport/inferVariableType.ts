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
    // Arrays become choice selectors
    return "choice";
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

  // For arrays, convert to comma-separated string for choice type
  if (Array.isArray(value) && type === "choice") {
    return value.join(", ");
  }

  // For non-null values, use as-is
  return value;
}
