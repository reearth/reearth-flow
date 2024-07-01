import { initialEdges, initialNodes } from "./nodeEdgeData";

export function generateWorkflows(count: number) {
  const workflows = [];
  for (let i = 0; i < count; i++) {
    const id = generateId();
    workflows.push({
      id,
      name: `Workflow ${id}`,
      nodes: initialNodes,
      edges: initialEdges,
    });
  }
  return workflows;
}

const generateId = () => {
  let result = "";
  const characters = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
  const charactersLength = characters.length;

  for (let i = 0; i < 4; i++) {
    result += characters.charAt(Math.floor(Math.random() * charactersLength));
  }

  return result;
};
