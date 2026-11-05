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

  const renderDialog = (
    onStart: (variables: AnyWorkflowVariable[]) => Promise<unknown> = vi.fn(
      async () => {},
    ),
    variables: AnyWorkflowVariable[] = deployed,
  ) => {
    const onDialogClose = vi.fn();
    render(
      <WorkflowVariablesRunDialog
        title="Run Deployment"
        description="For this run only."
        variables={variables}
        defaults={variables}
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

  const startButton = () => screen.getByRole("button", { name: /^Start/ });
  const editArea = () =>
    fireEvent.change(document.getElementById("default-0") as HTMLInputElement, {
      target: { value: "osaka" },
    });

  test("stays open with Start and Cancel disabled while the run is starting", async () => {
    let finish: (started: boolean) => void = () => {};
    const onStart = vi.fn(
      () => new Promise<boolean>((resolve) => (finish = resolve)),
    );
    const { onDialogClose } = renderDialog(onStart);

    fireEvent.click(startButton());

    expect(
      await screen.findByRole("button", { name: "Starting..." }),
    ).toBeDisabled();
    expect(screen.getByRole("button", { name: "Cancel" })).toBeDisabled();
    expect(onDialogClose).not.toHaveBeenCalled();

    finish(true);
    await waitFor(() => expect(onDialogClose).toHaveBeenCalled());
  });

  test("stays open with the edits when the run did not start", async () => {
    const onStart = vi.fn(async () => false);
    const { onDialogClose } = renderDialog(onStart);

    editArea();
    fireEvent.click(startButton());

    await waitFor(() => expect(startButton()).toBeEnabled());
    expect(onDialogClose).not.toHaveBeenCalled();
    expect(
      (document.getElementById("default-0") as HTMLInputElement).value,
    ).toBe("osaka");
  });

  test("stays open with the edits when starting the run fails", async () => {
    const error = vi.spyOn(console, "error").mockImplementation(() => {});
    const onStart = vi.fn(async () => {
      throw new Error("refused");
    });
    const { onDialogClose } = renderDialog(onStart);

    editArea();
    fireEvent.click(startButton());

    await waitFor(() => expect(startButton()).toBeEnabled());
    expect(onDialogClose).not.toHaveBeenCalled();
    expect(
      (document.getElementById("default-0") as HTMLInputElement).value,
    ).toBe("osaka");
    expect(error).toHaveBeenCalled();
    error.mockRestore();
  });

  test("shows a type it cannot edit as read-only, with no actions", () => {
    const folder = {
      ...variable("input", "gs://bucket/data"),
      type: "unsupported",
    } as AnyWorkflowVariable;
    renderDialog(undefined, [folder]);

    expect(screen.getByText("gs://bucket/data")).toBeInTheDocument();
    expect(screen.queryByText(/Unsupported variable type/)).toBeNull();
    expect(
      screen.queryByRole("button", { name: "Edit default value" }),
    ).toBeNull();
    expect(
      screen.queryByRole("button", { name: "Reset to default" }),
    ).toBeNull();
  });
});
