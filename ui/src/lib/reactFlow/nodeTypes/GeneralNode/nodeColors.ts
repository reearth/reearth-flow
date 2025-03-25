import { JobStatus } from "@flow/types";

export const getNodeColors = (type: string, status?: JobStatus) => {
  const baseColors = nodeColors[type as keyof typeof nodeColors];
  if (status) {
    return [
      nodeStatusColors[status],
      baseColors.selected,
      baseColors.selectedBackground,
    ];
  }
  return [
    baseColors.border,
    baseColors.selected,
    baseColors.selectedBackground,
  ];
};

const nodeColors = {
  reader: {
    border: "border-node-reader",
    selected: "border-node-reader-selected",
    selectedBackground: "bg-node-reader-selected",
  },
  writer: {
    border: "border-node-writer",
    selected: "border-node-writer-selected",
    selectedBackground: "bg-node-writer-selected",
  },
  transformer: {
    border: "border-node-transformer",
    selected: "border-node-transformer-selected",
    selectedBackground: "bg-node-transformer-selected",
  },
  subworkflow: {
    border: "border-node-subworkflow",
    selected: "border-node-subworkflow-selected",
    selectedBackground: "bg-node-subworkflow-selected",
  },
  default: {
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
