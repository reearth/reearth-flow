export type EmbedTheme = "dark" | "light";

export type EmbedWorkflowPathItem = {
  id: string;
  name: string;
};

export type EmbedErrorReason = "unavailable" | "corrupted";

export type EmbedInboundMessage =
  | { type: "flow:init"; token?: string; theme?: EmbedTheme }
  | { type: "flow:open-workflow"; workflowId: string };

export type EmbedOutboundMessage =
  | { type: "flow:ready" }
  | {
      type: "flow:loaded";
      projectId: string;
      timestamp: string;
      isEmpty: boolean;
    }
  | { type: "flow:workflow-changed"; path: EmbedWorkflowPathItem[] }
  | { type: "flow:error"; reason: EmbedErrorReason };

const isRecord = (value: unknown): value is Record<string, unknown> =>
  typeof value === "object" && value !== null;

const isTheme = (value: unknown): value is EmbedTheme =>
  value === "dark" || value === "light";

export const parseInboundMessage = (
  data: unknown,
): EmbedInboundMessage | undefined => {
  if (!isRecord(data)) return;

  switch (data.type) {
    case "flow:init": {
      if (data.token !== undefined && typeof data.token !== "string") return;
      if (data.theme !== undefined && !isTheme(data.theme)) return;
      return {
        type: "flow:init",
        token: data.token || undefined,
        theme: data.theme,
      };
    }
    case "flow:open-workflow": {
      if (typeof data.workflowId !== "string" || !data.workflowId) return;
      return { type: "flow:open-workflow", workflowId: data.workflowId };
    }
  }
};
