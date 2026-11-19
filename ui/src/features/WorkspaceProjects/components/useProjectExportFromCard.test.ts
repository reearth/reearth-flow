import { act, renderHook, waitFor } from "@testing-library/react";
import type * as Y from "yjs";

import { DEFAULT_ENTRY_GRAPH_ID } from "@flow/global-constants";
import type { Project } from "@flow/types";

import useProjectExportFromCard from "./useProjectExportFromCard";

// Stands in for y-websocket: records every document update it would have sent
// to the server while connected. Hoisted, since vi.mock is.
const { providers, FakeProvider } = vi.hoisted(() => {
  const providers: InstanceType<typeof FakeProvider>[] = [];
  class FakeProvider {
    sent: Uint8Array[] = [];
    private onSync?: () => void;
    private readonly handleUpdate = (update: Uint8Array) => {
      this.sent.push(update);
    };

    constructor(
      _url: string,
      _room: string,
      private readonly doc: Y.Doc,
    ) {
      doc.on("update", this.handleUpdate);
      providers.push(this);
    }

    once(event: string, callback: () => void) {
      if (event === "sync") this.onSync = callback;
    }

    sync() {
      this.onSync?.();
    }

    destroy() {
      this.doc.off("update", this.handleUpdate);
    }
  }
  return { providers, FakeProvider };
});

vi.mock("y-websocket", () => ({ WebsocketProvider: FakeProvider }));

vi.mock("@flow/config", () => ({
  config: () => ({ websocket: "ws://localhost" }),
}));

vi.mock("@flow/lib/auth", () => ({
  useAuth: () => ({ getAccessToken: async () => "token" }),
}));

const handleProjectExport = vi.fn(
  async (_input: { yDoc: Y.Doc | null; project?: Project }) => {},
);
vi.mock("@flow/hooks", () => ({
  useProjectExport: () => ({ handleProjectExport }),
}));

const project = { id: "project-1", name: "Project" } as Project;

describe("useProjectExportFromCard", () => {
  test("exporting a never-opened project sends nothing to the server", async () => {
    const { result } = renderHook(() => useProjectExportFromCard(project));

    await act(async () => {
      await result.current.handleProjectExportFromCard();
    });
    await waitFor(() => expect(providers).toHaveLength(1));
    const provider = providers[0];

    await act(async () => provider.sync());
    await waitFor(() => expect(handleProjectExport).toHaveBeenCalledTimes(1));

    expect(provider.sent).toHaveLength(0);
    // The exported copy still opens like a project that was initialised.
    const exported = handleProjectExport.mock.calls[0][0].yDoc;
    expect(exported?.getMap("metadata").get("initialized")).toBe(true);
    expect(exported?.getMap("workflows").has(DEFAULT_ENTRY_GRAPH_ID)).toBe(
      true,
    );
  });
});
