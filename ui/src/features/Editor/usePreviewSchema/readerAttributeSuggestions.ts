import type { Node, Workflow } from "@flow/types";

import type { AutocompleteSuggestion } from "../components/ParamsDialog/components/ValueEditorDialog/components/constants";

/**
 * Build the union of attribute-name autocomplete suggestions from every
 * reader node's probed schema across all workflows. Surfaced inside
 * `attributes["…"]` accessors in any FlowExpr field, since reader attributes
 * flow downstream to every action after them.
 */
export const buildReaderAttributeSuggestions = (
  workflows: Workflow[],
): AutocompleteSuggestion[] => {
  const seen = new Map<string, AutocompleteSuggestion>();

  workflows.forEach((workflow) => {
    (workflow.nodes as Node[] | undefined)?.forEach((node) => {
      if (node.type !== "reader") return;
      const ports = node.data.metadata?.schema?.ports;
      if (!ports) return;
      Object.values(ports).forEach((port) => {
        port.fields.forEach((field) => {
          if (seen.has(field.name)) return;
          seen.set(field.name, {
            label: field.name,
            insertText: field.name,
            type: "variable",
            detail:
              field.presence === "maybe"
                ? `${field.type} (optional)`
                : field.type,
          });
        });
      });
    });
  });

  return Array.from(seen.values());
};
