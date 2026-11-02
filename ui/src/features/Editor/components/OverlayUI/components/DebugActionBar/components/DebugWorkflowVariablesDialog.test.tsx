import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import type { AnyWorkflowVariable } from "@flow/types";

import DebugWorkflowVariablesDialog from "./DebugWorkflowVariablesDialog";

vi.mock("@flow/features/AssetsDialog", () => ({ default: () => null }));
vi.mock("@flow/features/CmsIntegrationDialog", () => ({ default: () => null }));

const variable = (
  name: string,
  defaultValue: any,
  required: boolean,
): AnyWorkflowVariable =>
  ({
    id: name,
    name,
    type: "text",
    defaultValue,
    required,
    public: true,
  }) as AnyWorkflowVariable;

const renderDialog = (variables: AnyWorkflowVariable[]) => {
  const onDebugRunStart = vi.fn(async () => {});
  render(
    <DebugWorkflowVariablesDialog
      debugRunWorkflowVariables={variables}
      workflowVariableDefaults={variables}
      onDebugRunStart={onDebugRunStart}
      onDialogClose={vi.fn()}
    />,
  );
  return { onDebugRunStart };
};

const startButton = () => screen.getByRole("button", { name: "Start" });

describe("DebugWorkflowVariablesDialog", () => {
  test("blocks the run and names required variables that have no value", () => {
    const { onDebugRunStart } = renderDialog([
      variable("CITY", "Tokyo", true),
      variable("API_KEY", "", true),
      variable("OPTIONAL", "", false),
    ]);

    expect(startButton()).toBeDisabled();
    const message = screen.getByText(/required variables to start/);
    expect(message).toHaveTextContent(/API_KEY$/);

    fireEvent.click(startButton());
    expect(onDebugRunStart).not.toHaveBeenCalled();
  });

  test("allows the run once every required variable has a value", () => {
    const vars = [variable("CITY", "Tokyo", true), variable("OPT", "", false)];
    const { onDebugRunStart } = renderDialog(vars);

    expect(startButton()).toBeEnabled();
    fireEvent.click(startButton());
    expect(onDebugRunStart).toHaveBeenCalledWith(vars);
  });
});
