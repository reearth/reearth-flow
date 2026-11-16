import { fireEvent, render, screen } from "@testing-library/react";
import { vi } from "vitest";

import type { EditorContext } from "@flow/components/SchemaForm";

import FlowExprEditorDialog from ".";
import type { CodeValue } from ".";

const ASSET_URL = "https://example.com/assets/data.gml";

vi.mock("@flow/lib/i18n", () => ({
  useT: () => (key: string) => key,
}));

vi.mock("@flow/stores", () => ({
  useCurrentProject: () => [{ id: "project-1" }],
}));

vi.mock("@flow/lib/gql", () => ({
  useWorkflowVariables: () => ({
    useGetWorkflowVariables: () => ({ workflowVariables: [] }),
  }),
}));

vi.mock("@flow/features/AssetsDialog", () => ({
  default: ({ onAssetSelect }: { onAssetSelect: (asset: any) => void }) => (
    <button onClick={() => onAssetSelect({ url: ASSET_URL })}>
      pick asset
    </button>
  ),
}));

vi.mock("@flow/features/CmsIntegrationDialog", () => ({
  default: ({ onCmsItemValue }: { onCmsItemValue: (url: string) => void }) => (
    <button onClick={() => onCmsItemValue(ASSET_URL)}>pick cms item</button>
  ),
}));

beforeAll(() => {
  globalThis.ResizeObserver = class {
    observe() {}
    unobserve() {}
    disconnect() {}
  } as unknown as typeof ResizeObserver;
  window.Element.prototype.scrollIntoView = () => {};
});

const fieldContext = (value: CodeValue): EditorContext => ({
  key: "dataset",
  name: "dataset",
  path: ["dataset"],
  value,
  schema: {
    title: "File Path",
    format: "code",
    properties: {
      type: { type: "string", enum: ["flowExpr", "string"] },
      value: { type: "string" },
    },
  },
  fieldName: "dataset",
});

const submitted = (initial: CodeValue, pick: string) => {
  const onValueSubmit = vi.fn();
  render(
    <FlowExprEditorDialog
      open
      fieldContext={fieldContext(initial)}
      onClose={() => {}}
      onValueSubmit={onValueSubmit}
    />,
  );
  fireEvent.click(
    screen.getByText(pick === "asset" ? "Workspace Assets" : "CMS Integration"),
  );
  fireEvent.click(
    screen.getByText(pick === "asset" ? "pick asset" : "pick cms item"),
  );
  fireEvent.click(screen.getByText("Apply"));
  return onValueSubmit.mock.calls[0][0] as CodeValue;
};

describe("FlowExprEditorDialog URL insertion", () => {
  it.each(["asset", "cms"])(
    "inserts a %s URL as a string literal on the Expression tab",
    (pick) => {
      expect(submitted({ type: "flowExpr", value: "" }, pick)).toEqual({
        type: "flowExpr",
        value: JSON.stringify(ASSET_URL),
      });
    },
  );

  it.each(["asset", "cms"])(
    "inserts a %s URL as-is on the Literal string tab",
    (pick) => {
      expect(submitted({ type: "string", value: "" }, pick)).toEqual({
        type: "string",
        value: ASSET_URL,
      });
    },
  );
});
