import { load as yamlLoad } from "js-yaml";

import {
  validateEngineReadyWorkflow,
  validateEngineReadyGraph,
} from "./jsonValidation";

export const validateWorkflowYaml = (
  yamlString: string,
): { isValid: boolean; error?: string } => {
  let parsedYaml: any;

  try {
    parsedYaml = yamlLoad(yamlString);
  } catch (e) {
    return {
      isValid: false,
      error: `Invalid YAML format: ${(e as Error).message}`,
    };
  }

  // Check if the parsed YAML is null or undefined
  if (parsedYaml == null) {
    return {
      isValid: false,
      error: "YAML content is empty or contains only comments",
    };
  }

  // Use the existing validation logic since the structure should be the same
  if (!validateEngineReadyWorkflow(parsedYaml)) {
    return {
      isValid: false,
      error: "YAML does not match EngineReadyWorkflow structure",
    };
  }

  return { isValid: true };
};

// Helper function to validate a YAML file specifically for a single graph
export const validateGraphYaml = (
  yamlString: string,
): { isValid: boolean; error?: string } => {
  let parsedYaml: any;

  try {
    parsedYaml = yamlLoad(yamlString);
  } catch (e) {
    return {
      isValid: false,
      error: `Invalid YAML format: ${(e as Error).message}`,
    };
  }

  if (parsedYaml == null) {
    return {
      isValid: false,
      error: "YAML content is empty or contains only comments",
    };
  }

  if (!validateEngineReadyGraph(parsedYaml)) {
    return {
      isValid: false,
      error: "YAML does not match EngineReadyGraph structure",
    };
  }

  return { isValid: true };
};
