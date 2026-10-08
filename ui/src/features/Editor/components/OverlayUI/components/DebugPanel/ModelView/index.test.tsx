import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import type * as IntermediateDataViewModule from "@flow/lib/gql/intermediateDataView";
import { IntermediateDataViewError } from "@flow/lib/gql/intermediateDataView/errors";
import type { IntermediateDataViewState } from "@flow/lib/gql/intermediateDataView/useApi";
import type { IntermediateDataView } from "@flow/types";

import ModelView from "./index";

let state: IntermediateDataViewState = { isRendering: false };

vi.mock("@flow/lib/gql/intermediateDataView", async (importOriginal) => ({
  ...(await importOriginal<typeof IntermediateDataViewModule>()),
  useIntermediateDataView: () => ({
    useView: () => state,
    requestView: vi.fn(),
    errorMessage: (error: IntermediateDataViewError) => `message:${error.kind}`,
  }),
}));

// WebGL is not available here; the viewer only has to be handed the model.
vi.mock("@flow/components/visualizations/GlbViewer", () => ({
  default: ({ url }: { url: string }) => <div>model at {url}</div>,
  BACK_FACE_COLOR: "#e5484d",
}));

const request = { jobId: "job1", fileId: "n1.default", row: 2 };

const view = (
  overrides: Partial<IntermediateDataView>,
): IntermediateDataView => ({
  id: "gltf-r2-abc",
  jobId: "job1",
  fileId: "n1.default",
  shape: "gltf",
  status: "ready",
  ...overrides,
});

const setup = (openError?: IntermediateDataViewError) => {
  const onRetry = vi.fn();
  const onBack = vi.fn();
  render(
    <ModelView
      request={request}
      openError={openError}
      onRetry={onRetry}
      onBack={onBack}
    />,
  );
  return { onRetry, onBack };
};

const retryButton = () => screen.queryByRole("button", { name: "Try again" });

beforeEach(() => {
  state = { isRendering: false };
});

describe("ModelView", () => {
  test("shows a running render and how long it has taken", () => {
    state = { isRendering: true, renderStartedAt: Date.now() - 12_000 };
    setup();

    expect(screen.getByText(/Rendering the 3D model/)).toBeDefined();
    expect(screen.getByText("0:12")).toBeDefined();
  });

  test("shows the model once it is ready", () => {
    state = {
      isRendering: false,
      view: view({ format: "glb", entryPointUrl: "https://api/v/m.glb" }),
    };
    setup();

    expect(screen.getByText("model at https://api/v/m.glb")).toBeDefined();
    // The red the viewer draws back faces in is a finding, so it is explained.
    expect(screen.getByText(/Back of a face/)).toBeDefined();
  });

  test("says why there is nothing to show, without offering to ask again", () => {
    state = {
      isRendering: false,
      view: view({
        status: "empty",
        error: "Row 2 carried no geometry the view draws",
      }),
    };
    setup();

    expect(screen.getByText(/no geometry a 3D model can show/)).toBeDefined();
    expect(
      screen.getByText("Row 2 carried no geometry the view draws"),
    ).toBeDefined();
    expect(retryButton()).toBeNull();
  });

  test("explains a 2D row", () => {
    state = {
      isRendering: false,
      view: view({ status: "unsupportedGeometry" }),
    };
    setup();

    expect(screen.getByText(/is 2D, so it has no 3D model/)).toBeDefined();
    expect(retryButton()).toBeNull();
  });

  test("offers to try a failed render again", () => {
    state = {
      isRendering: false,
      view: view({ status: "failed", error: "the render stopped" }),
    };
    const { onRetry } = setup();

    fireEvent.click(screen.getByRole("button", { name: "Try again" }));

    expect(onRetry).toHaveBeenCalledTimes(1);
  });

  test("offers to try again after a render that ran out of time", () => {
    state = {
      isRendering: false,
      error: new IntermediateDataViewError("timedOut"),
    };
    setup();

    expect(screen.getByText("message:timedOut")).toBeDefined();
    expect(retryButton()).not.toBeNull();
  });

  test("does not offer to try again when asking again cannot help", () => {
    state = {
      isRendering: false,
      error: new IntermediateDataViewError("permissionDenied"),
    };
    setup();

    expect(screen.getByText("message:permissionDenied")).toBeDefined();
    expect(retryButton()).toBeNull();
  });

  test("says when too many renders are running to start another", () => {
    setup(new IntermediateDataViewError("tooManyRenders"));

    expect(screen.getByText("message:tooManyRenders")).toBeDefined();
    expect(retryButton()).not.toBeNull();
  });

  test("goes back to the row's details", () => {
    const { onBack } = setup();

    fireEvent.click(screen.getAllByRole("button")[0]);

    expect(onBack).toHaveBeenCalledTimes(1);
  });
});
