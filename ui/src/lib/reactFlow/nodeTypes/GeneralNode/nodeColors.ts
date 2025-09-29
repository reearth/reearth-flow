import { JobStatus } from "@flow/types";

export const getNodeColors = (
  type: string,
  disabled?: boolean,
  status?: JobStatus,
) => {
  const baseColors = nodeColors[type as keyof typeof nodeColors];
  if (disabled) {
    return [
      nodeColors.disabled.background,
      nodeColors.disabled.border,
      nodeColors.disabled.selected,
      nodeColors.disabled.selectedBackground,
    ];
  }
  if (status) {
    return [
      nodeStatusColors[status],
      baseColors.selected,
      baseColors.selectedBackground,
    ];
  }
  return [
    baseColors.border,
    baseColors.background,
    baseColors.selected,
    baseColors.selectedBackground,
  ];
};

const nodeColors = {
  reader: {
    background: "bg-node-reader",
    border: "border-node-reader",
    selected: "border-node-reader-selected",
    selectedBackground: "bg-node-reader-selected",
  },
  writer: {
    background: "bg-node-writer",
    border: "border-node-writer",
    selected: "border-node-writer-selected",
    selectedBackground: "bg-node-writer-selected",
  },
  transformer: {
    background: "bg-node-transformer",
    border: "border-node-transformer",
    selected: "border-node-transformer-selected",
    selectedBackground: "bg-node-transformer-selected",
  },
  subworkflow: {
    background: "bg-node-subworkflow",
    border: "border-node-subworkflow",
    selected: "border-node-subworkflow-selected",
    selectedBackground: "bg-node-subworkflow-selected",
  },
  disabled: {
    background: "bg-node-zinc-600",
    border: "border-primary/20",
    selected: "border-zinc-600",
    selectedBackground: "bg-zinc-600",
  },
  default: {
    background: "bg-node-zinc-600",
    border: "border-primary/20",
    selected: "border-zinc-600",
    selectedBackground: "bg-zinc-600",
  },
};

export const nodeStatusColors: Record<JobStatus | "default", string> = {
  completed: "border-success",
  failed: "border-destructive",
  running: "active-node-status-border",
  queued: "queued-node-status-border",
  cancelled: "border-warning",
  // processing: "active-node-status-border",
  // pending: "queued-node-status-border",
  // starting: "queued-node-status-border",
  default: "",
};
