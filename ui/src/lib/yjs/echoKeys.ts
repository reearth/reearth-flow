/**
 * Keys for the awareness echo channel (`AwarenessUser.activeElements`).
 *
 * Central so both sides of a key agree — a dialog that broadcasts its own key
 * and a component elsewhere reading it would otherwise drift apart silently.
 */
export const ECHO_KEYS = {
  actionsDropdown: "dropdown:actions",
  homeMenu: "dropdown:home",
  workflowsDropdown: "dropdown:workflows",
  debugStartPopover: "popover:debug-start",
  debugRunsPopover: "popover:debug-runs",
  debugVariablesDialog: "popover:debug-variables",
} as const;

export const actionItemEchoKey = (item: string) => `action-item:${item}`;
export const workflowItemEchoKey = (workflowId: string) =>
  `workflow-item:${workflowId}`;
export const versionRowEchoKey = (snapshotNumber: number) =>
  `version-row:${snapshotNumber}`;
export const debugVariableEchoKey = (variableId: string) =>
  `debug-var:${variableId}`;
