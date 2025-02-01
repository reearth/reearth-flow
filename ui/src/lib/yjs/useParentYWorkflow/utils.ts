import { Node, PseudoPort } from "@flow/types";

// Function to update and return pseudoInputs or pseudoOutputs
export function getUpdatedPseudoPortsParam(
  prevPseudoPorts: PseudoPort[],
  newPseudoPort: PseudoPort,
): PseudoPort[] {
  const portIndex = prevPseudoPorts.findIndex(
    (port) => port.nodeId === newPseudoPort.nodeId,
  );

  // If the pseudoInput/Output already exists, we want to update it. Otherwise, we want to add it.
  const updatedPseudoPorts =
    portIndex !== -1
      ? prevPseudoPorts.map((port, idx) =>
          idx === portIndex ? newPseudoPort : port,
        )
      : [...prevPseudoPorts, newPseudoPort];

  return updatedPseudoPorts;
}

export function splitPorts(
  ports: { nodeId: string; portName: string }[],
  nodeToDelete: Node,
) {
  const portToRemove = ports.find((port) => port.nodeId === nodeToDelete.id);
  const portsToUpdate = ports.filter((port) => port.nodeId !== nodeToDelete.id);

  return { portToRemove, portsToUpdate };
}
