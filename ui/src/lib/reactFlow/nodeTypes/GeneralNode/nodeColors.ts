export const getNodeColors = (type: string) =>
  Object.keys(nodeColors).includes(type)
    ? Object.values(nodeColors[type as keyof typeof nodeColors])
    : Object.values(nodeColors.default);

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
