export const typeColorClass = (type: string) => {
  switch (type) {
    case "transformer":
      return "bg-node-transformer/95 dark:bg-node-transformer/60";
    case "reader":
      return "bg-node-reader/95 dark:bg-node-reader/60";
    case "writer":
      return "bg-node-writer/85 dark:bg-node-writer/30";
    default:
      return "bg-secondary";
  }
};
