import { act, renderHook } from "@testing-library/react";
import {
  Awareness,
  applyAwarenessUpdate,
  encodeAwarenessUpdate,
} from "y-protocols/awareness";
import * as Y from "yjs";

import type { WorkflowVariable } from "@flow/types";

import useWorkflowVarSession, {
  WORKFLOW_VAR_SESSION_FIELD,
  readWorkflowVarSession,
} from "./useWorkflowVarSession";

const variable = (id: string, name: string): WorkflowVariable => ({
  id,
  name,
  defaultValue: "",
  type: "text",
  required: false,
  public: true,
});

// One client: its document, its awareness, and the document updates it made.
const createClient = (clientID: number) => {
  const doc = new Y.Doc();
  doc.clientID = clientID;
  const awareness = new Awareness(doc);
  const docUpdates = vi.fn();
  doc.on("update", docUpdates);
  return { doc, awareness, docUpdates };
};

type Client = ReturnType<typeof createClient>;

// Forwards every awareness change as it happens, as the provider does.
const connect = (a: Client, b: Client) => {
  const forward =
    (from: Awareness, to: Awareness) =>
    ({
      added,
      updated,
      removed,
    }: {
      added: number[];
      updated: number[];
      removed: number[];
    }) =>
      applyAwarenessUpdate(
        to,
        encodeAwarenessUpdate(from, [...added, ...updated, ...removed]),
        "remote",
      );
  a.awareness.on("update", forward(a.awareness, b.awareness));
  b.awareness.on("update", forward(b.awareness, a.awareness));
  applyAwarenessUpdate(
    b.awareness,
    encodeAwarenessUpdate(a.awareness, [a.doc.clientID]),
    "remote",
  );
  applyAwarenessUpdate(
    a.awareness,
    encodeAwarenessUpdate(b.awareness, [b.doc.clientID]),
    "remote",
  );
};

const ownCopy = (client: Client) =>
  client.awareness.getLocalState()?.[WORKFLOW_VAR_SESSION_FIELD];

describe("useWorkflowVarSession", () => {
  test("joining publishes nothing until someone edits", () => {
    const a = createClient(1);
    const { result } = renderHook(() => useWorkflowVarSession(a.awareness));

    expect(result.current.session).toBeUndefined();
    expect(ownCopy(a)).toBeUndefined();
    expect(a.docUpdates).not.toHaveBeenCalled();
  });

  test("an edit reaches the others without touching the document", () => {
    const a = createClient(1);
    const b = createClient(2);
    connect(a, b);
    const hookA = renderHook(() => useWorkflowVarSession(a.awareness));
    const hookB = renderHook(() => useWorkflowVarSession(b.awareness));

    const vars = [variable("v1", "alpha")];
    act(() =>
      hookA.result.current.publishSession({ variables: vars, base: [] }),
    );

    expect(hookB.result.current.session?.variables).toEqual(vars);
    expect(a.docUpdates).not.toHaveBeenCalled();
    expect(b.docUpdates).not.toHaveBeenCalled();
  });

  test("the session outlives the client that published it", () => {
    const a = createClient(1);
    const b = createClient(2);
    connect(a, b);
    const hookA = renderHook(() => useWorkflowVarSession(a.awareness));
    const hookB = renderHook(() => useWorkflowVarSession(b.awareness));

    const vars = [variable("v1", "alpha")];
    act(() => {
      hookA.result.current.publishSession({ variables: vars, base: [] });
      // Leaving straight after publishing, as cancel and save do.
      hookA.result.current.leaveSession();
    });

    expect(ownCopy(b)?.variables).toEqual(vars);
    expect(hookB.result.current.session?.variables).toEqual(vars);
  });

  test("it ends with its last participant", () => {
    const a = createClient(1);
    const b = createClient(2);
    connect(a, b);
    const hookA = renderHook(() => useWorkflowVarSession(a.awareness));
    const hookB = renderHook(() => useWorkflowVarSession(b.awareness));

    act(() =>
      hookA.result.current.publishSession({
        variables: [variable("v1", "alpha")],
        base: [],
      }),
    );
    act(() => {
      hookA.result.current.leaveSession();
      hookB.result.current.leaveSession();
    });

    expect(readWorkflowVarSession(a.awareness)).toBeUndefined();
    expect(readWorkflowVarSession(b.awareness)).toBeUndefined();
  });

  test("each edit supersedes the newest revision, whoever published it", () => {
    const a = createClient(1);
    const b = createClient(2);
    connect(a, b);
    const hookA = renderHook(() => useWorkflowVarSession(a.awareness));
    const hookB = renderHook(() => useWorkflowVarSession(b.awareness));

    act(() =>
      hookA.result.current.publishSession({
        variables: [variable("v1", "alpha")],
        base: [],
      }),
    );
    const later = [variable("v1", "beta")];
    act(() =>
      hookB.result.current.publishSession({ variables: later, base: [] }),
    );

    expect(hookA.result.current.session?.variables).toEqual(later);
    expect(hookA.result.current.session?.rev).toBe(2);
  });
});
