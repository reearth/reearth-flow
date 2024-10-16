import YAML from "yaml";

import { Workflow } from "@flow/types";

import { consolidateWorkflows } from "./consolidateWorkflows";

export const createWorkflowsYaml = (workflows?: Workflow[]) => {
  if (!workflows) return;
  const yamlReadyWorkflow = consolidateWorkflows(workflows);

  const yamlWorkflow = YAML.stringify(yamlReadyWorkflow);

  return { workflowId: yamlReadyWorkflow.id, yamlWorkflow };
};

// const exampleWorkflows: Workflow[] = [
//   {
//     id: "main",
//     name: "Main Workflow",
//     nodes: [
//       {
//         id: "dZSVTDV68jVSsyUZK8dyqvOR",
//         type: "transformer",
//         position: {
//           x: 1090,
//           y: 337,
//         },
//         data: {
//           name: "AreaOnAreaOverlayer",
//           inputs: ["default"],
//           outputs: ["area", "remnants", "rejected"],
//           status: "idle",
//           locked: false,
//         },
//         measured: {
//           width: 150,
//           height: 25,
//         },
//         selected: false,
//         dragging: false,
//       },
//       {
//         id: "Mb9n0r2PiBYrFYKbMCLzaJkp",
//         type: "reader",
//         position: {
//           x: 329.8883307877826,
//           y: 267.52389978853313,
//         },
//         data: {
//           name: "FeatureCreator",
//           inputs: [],
//           outputs: ["default"],
//           status: "idle",
//           locked: false,
//         },
//         measured: {
//           width: 150,
//           height: 25,
//         },
//         selected: false,
//         dragging: false,
//       },
//       {
//         id: "1-workflow",
//         type: "subworkflow",
//         position: {
//           x: 721.3289103618781,
//           y: 309.7737760416992,
//         },
//         data: {
//           name: "Sub Workflow-1",
//           status: "idle",
//           inputs: ["source"],
//           outputs: ["target"],
//         },
//         measured: {
//           width: 150,
//           height: 25,
//         },
//         selected: true,
//         dragging: false,
//       },
//     ],
//     edges: [
//       {
//         source: "Mb9n0r2PiBYrFYKbMCLzaJkp",
//         sourceHandle: "default",
//         target: "1-workflow",
//         id: "xy-edge__Mb9n0r2PiBYrFYKbMCLzaJkpdefault-1-workflow",
//       },
//       {
//         source: "1-workflow",
//         target: "dZSVTDV68jVSsyUZK8dyqvOR",
//         targetHandle: "default",
//         id: "xy-edge__1-workflow-dZSVTDV68jVSsyUZK8dyqvORdefault",
//       },
//     ],
//   },
//   {
//     id: "1-workflow",
//     name: "Sub Workflow-1",
//     nodes: [
//       {
//         id: "Cqq9X6kScznZxNQpgI0LmRle",
//         type: "entrance",
//         position: {
//           x: 200,
//           y: 200,
//         },
//         data: {
//           name: "New Entrance node",
//           outputs: ["target"],
//           status: "idle",
//         },
//         measured: {
//           width: 34,
//           height: 50,
//         },
//       },
//       {
//         id: "m7xQtMSe4gJzPV4c8F0d5jT7",
//         type: "exit",
//         position: {
//           x: 1000,
//           y: 200,
//         },
//         data: {
//           name: "New Exit node",
//           inputs: ["source"],
//           status: "idle",
//         },
//         measured: {
//           width: 34,
//           height: 50,
//         },
//       },
//       {
//         id: "3wiVciHlT6flIb03bAkkwM6a",
//         type: "transformer",
//         position: {
//           x: 600,
//           y: 240,
//         },
//         data: {
//           name: "AreaOnAreaOverlayer",
//           inputs: ["default"],
//           outputs: ["area", "remnants", "rejected"],
//           status: "idle",
//           locked: false,
//         },
//         measured: {
//           width: 150,
//           height: 25,
//         },
//       },
//     ],
//     edges: [
//       {
//         source: "Cqq9X6kScznZxNQpgI0LmRle",
//         target: "3wiVciHlT6flIb03bAkkwM6a",
//         targetHandle: "default",
//         id: "xy-edge__Cqq9X6kScznZxNQpgI0LmRle-3wiVciHlT6flIb03bAkkwM6adefault",
//       },
//       {
//         source: "3wiVciHlT6flIb03bAkkwM6a",
//         sourceHandle: "remnants",
//         target: "m7xQtMSe4gJzPV4c8F0d5jT7",
//         targetHandle: "source",
//         id: "xy-edge__3wiVciHlT6flIb03bAkkwM6aremnants-m7xQtMSe4gJzPV4c8F0d5jT7source",
//       },
//     ],
//   },
// ];
