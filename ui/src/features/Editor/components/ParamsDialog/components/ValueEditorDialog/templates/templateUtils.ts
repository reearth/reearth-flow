import type { ExpressionTemplate } from "./templateData";

export type PlaceholderValue = {
  key: string;
  value: string;
};

export type ProcessedTemplate = {
  populatedCode: string;
  hasPlaceholders: boolean;
  placeholders: {
    key: string;
    description: string;
    defaultValue?: string;
    currentValue: string;
  }[];
};

/**
 * Extract placeholder keys from template code
 * Matches patterns like {{variableName}}
 */
export function extractPlaceholders(code: string): string[] {
  const placeholderRegex = /\{\{([^}]+)\}\}/g;
  const placeholders: string[] = [];
  let match;

  while ((match = placeholderRegex.exec(code)) !== null) {
    const key = match[1].trim();
    if (!placeholders.includes(key)) {
      placeholders.push(key);
    }
  }

  return placeholders;
}

/**
 * Replace placeholders in template code with actual values
 */
export function replacePlaceholders(
  code: string,
  values: PlaceholderValue[],
): string {
  let result = code;

  for (const { key, value } of values) {
    const placeholder = `{{${key}}}`;
    result = result.replace(
      new RegExp(placeholder.replace(/[{}]/g, "\\$&"), "g"),
      value,
    );
  }

  return result;
}

/**
 * Process a template with placeholder values
 */
export function processTemplate(
  template: ExpressionTemplate,
  placeholderValues: PlaceholderValue[] = [],
): ProcessedTemplate {
  const extractedPlaceholders = extractPlaceholders(template.rhaiCode);
  const hasPlaceholders = extractedPlaceholders.length > 0;

  // Create placeholder objects with current values
  const placeholders = template.placeholders.map((placeholder) => {
    const currentValue =
      placeholderValues.find((v) => v.key === placeholder.key)?.value ||
      placeholder.defaultValue ||
      "";

    return {
      ...placeholder,
      currentValue,
    };
  });

  // Replace placeholders in the code
  const populatedCode = replacePlaceholders(
    template.rhaiCode,
    placeholders.map((p) => ({ key: p.key, value: p.currentValue })),
  );

  return {
    populatedCode,
    hasPlaceholders,
    placeholders,
  };
}

/**
 * Validate placeholder values
 */
export function validatePlaceholderValue(
  key: string,
  value: string,
  template: ExpressionTemplate,
): { isValid: boolean; error?: string } {
  const placeholder = template.placeholders.find((p) => p.key === key);

  if (!placeholder) {
    return { isValid: false, error: "Unknown placeholder" };
  }

  // Basic validation - can be extended later
  if (value.trim() === "") {
    return { isValid: false, error: "Value cannot be empty" };
  }

  // Validate based on placeholder context
  if (
    key.toLowerCase().includes("attribute") &&
    !value.match(/^[a-zA-Z][a-zA-Z0-9:_-]*$/)
  ) {
    return { isValid: false, error: "Invalid attribute name format" };
  }

  if (key.toLowerCase().includes("filename") && value.includes("/")) {
    return {
      isValid: false,
      error: "Filename should not contain path separators",
    };
  }

  return { isValid: true };
}

/**
 * Generate a preview of the template with current placeholder values
 */
export function generateTemplatePreview(
  template: ExpressionTemplate,
  placeholderValues: PlaceholderValue[],
): string {
  const processed = processTemplate(template, placeholderValues);

  // If no placeholders or all are filled, return the populated code
  if (
    !processed.hasPlaceholders ||
    processed.placeholders.every((p) => p.currentValue)
  ) {
    return processed.populatedCode;
  }

  // Otherwise, return the template with unfilled placeholders highlighted
  let preview = template.rhaiCode;
  for (const placeholder of processed.placeholders) {
    const placeholderText = `{{${placeholder.key}}}`;
    const regex = new RegExp(placeholderText.replace(/[{}]/g, "\\$&"), "g");

    if (!placeholder.currentValue) {
      preview = preview.replace(regex, `[${placeholder.key.toUpperCase()}]`);
    } else {
      preview = preview.replace(regex, placeholder.currentValue);
    }
  }

  return preview;
}

/**
 * Check if template is ready to be inserted (all required placeholders filled)
 */
export function isTemplateReady(
  template: ExpressionTemplate,
  placeholderValues: PlaceholderValue[],
): boolean {
  const processed = processTemplate(template, placeholderValues);
  return processed.placeholders.every((p) => p.currentValue.trim() !== "");
}

/**
 * Format template code for display with syntax highlighting hints
 */
export function formatTemplateCode(code: string): string {
  // Add line breaks for better readability in multi-line templates
  return code
    .replace(/;\s*/g, ";\n")
    .replace(/\{\s*/g, "{\n  ")
    .replace(/\s*\}/g, "\n}")
    .replace(/\n\s*\n/g, "\n"); // Remove extra blank lines
}
