import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import type { AnyWorkflowVariable } from "@flow/types";

import WorkflowVariablesRunDialog from ".";

vi.mock("@flow/features/AssetsDialog", () => ({ default: () => null }));
vi.mock("@flow/features/CmsIntegrationDialog", () => ({ default: () => null }));

const variable = (name: string, defaultValue: any): AnyWorkflowVariable =>
  ({
    id: name,
    name,
    type: "text",
    defaultValue,
    required: false,
    public: true,
  }) as AnyWorkflowVariable;

describe("WorkflowVariablesRunDialog", () => {
  const deployed = [variable("area", "tokyo"), variable("lang", "ja")];

  const renderDialog = () => {
    const onStart = vi.fn(async () => {});
    const onDialogClose = vi.fn();
    render(
      <WorkflowVariablesRunDialog
        title="Run Deployment"
        description="For this run only."
        variables={deployed}
        defaults={deployed}
        onStart={onStart}
        onDialogClose={onDialogClose}
      />,
    );
    return { onStart, onDialogClose };
  };

  test("shows the given title and description", () => {
    renderDialog();
    expect(screen.getByText("Run Deployment")).toBeInTheDocument();
    expect(screen.getByText("For this run only.")).toBeInTheDocument();
  });

  test("starts with the edited values and closes", async () => {
    const { onStart, onDialogClose } = renderDialog();

    fireEvent.change(document.getElementById("default-0") as HTMLInputElement, {
      target: { value: "osaka" },
    });
    fireEvent.click(screen.getByRole("button", { name: "Start" }));

    await waitFor(() => expect(onDialogClose).toHaveBeenCalled());
    expect(onStart).toHaveBeenCalledWith([
      { ...deployed[0], defaultValue: "osaka" },
      deployed[1],
    ]);
  });

  test("resetting returns a value to its default", async () => {
    const { onStart } = renderDialog();

    fireEvent.change(document.getElementById("default-0") as HTMLInputElement, {
      target: { value: "osaka" },
    });
    const [reset] = screen.getAllByRole("button", { name: "Reset to default" });
    fireEvent.click(reset);
    fireEvent.click(screen.getByRole("button", { name: "Start" }));

    await waitFor(() => expect(onStart).toHaveBeenCalledWith(deployed));
  });
});
