import { act, renderHook } from "@testing-library/react";
import { useState } from "react";
import type { Awareness } from "y-protocols/awareness";

import { DEFAULT_ENTRY_GRAPH_ID as MAIN } from "@flow/global-constants";
import type { AwarenessUser, Workflow } from "@flow/types";

import useSpotlightUser from "./useSpotlightUser";
import useWorkflowTabs from "./useWorkflowTabs";

vi.mock("@xyflow/react", () => ({
  useReactFlow: () => ({ setViewport: vi.fn() }),
}));

const workflow = (id: string, subworkflowIds: string[] = []): Workflow => ({
  id,
  name: id,
  nodes: subworkflowIds.map((subworkflowId) => ({
    id: `node-${subworkflowId}`,
    type: "subworkflow",
    position: { x: 0, y: 0 },
    data: { officialName: "Subworkflow", subworkflowId },
  })),
  edges: [],
});

// main → a → b
const rawWorkflows = [
  workflow(MAIN, ["a"]),
  workflow("a", ["b"]),
  workflow("b"),
];

const REMOTE = 2;
const remote = (currentWorkflowId: string, openWorkflowIds: string[]) => ({
  [REMOTE]: {
    clientId: REMOTE,
    currentWorkflowId,
    openWorkflowIds,
  } as AwarenessUser,
});

const yAwareness = { setLocalStateField: vi.fn() } as unknown as Awareness;

const renderFollower = () =>
  renderHook(
    ({ users }: { users: Record<string, AwarenessUser> }) => {
      const [currentWorkflowId, setCurrentWorkflowId] = useState(MAIN);
      const tabs = useWorkflowTabs({
        currentWorkflowId,
        rawWorkflows,
        setCurrentWorkflowId,
      });
      const spotlight = useSpotlightUser({
        yAwareness,
        users,
        currentWorkflowId,
        openWorkflowIds: tabs.openWorkflowIds,
        handleWorkflowOpen: tabs.handleWorkflowOpen,
        handleWorkflowClose: tabs.handleWorkflowClose,
        getWorkflowLineage: tabs.getWorkflowLineage,
      });
      return { ...tabs, ...spotlight };
    },
    { initialProps: { users: remote(MAIN, [MAIN]) } },
  );

describe("useSpotlightUser closing followed workflows", () => {
  it("closes a workflow the spotlight opened once they close it", () => {
    const { result, rerender } = renderFollower();
    act(() => result.current.handleSpotlightUserSelect(REMOTE));
    rerender({ users: remote("a", [MAIN, "a"]) });
    expect(result.current.openWorkflowIds).toEqual([MAIN, "a"]);

    rerender({ users: remote(MAIN, [MAIN]) });
    expect(result.current.openWorkflowIds).toEqual([MAIN]);
  });

  it("keeps it open when the local user opened a workflow nested in it", () => {
    const { result, rerender } = renderFollower();
    act(() => result.current.handleSpotlightUserSelect(REMOTE));
    rerender({ users: remote("a", [MAIN, "a"]) });
    act(() => result.current.handleWorkflowOpen("b"));
    expect(result.current.openWorkflowIds).toEqual([MAIN, "a", "b"]);

    rerender({ users: remote(MAIN, [MAIN]) });
    expect(result.current.openWorkflowIds).toEqual([MAIN, "a", "b"]);
  });
});
