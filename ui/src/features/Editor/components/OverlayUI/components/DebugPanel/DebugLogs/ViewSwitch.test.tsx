import { render, screen } from "@testing-library/react";
import { describe, expect, test, vi } from "vitest";

import ViewSwitch from "@flow/features/Editor/components/OverlayUI/components/DebugPanel/DebugLogs/ViewSwitch";

describe("ViewSwitch", () => {
  const base = { view: "logs" as const, onViewChange: vi.fn() };

  test("disables the bug button and shows no badge at zero", () => {
    render(<ViewSwitch {...base} diagnosticsCount={0} hasBlocking={false} />);
    const buttons = screen.getAllByRole("button");
    expect(buttons[1]).toBeDisabled();
    expect(screen.queryByText("0")).not.toBeInTheDocument();
  });

  test("enables it and badges the count when there are diagnostics", () => {
    render(<ViewSwitch {...base} diagnosticsCount={7} hasBlocking />);
    expect(screen.getAllByRole("button")[1]).not.toBeDisabled();
    expect(screen.getByText("7")).toBeInTheDocument();
  });

  test("caps the badge so a big count cannot blow out the toolbar", () => {
    render(
      <ViewSwitch {...base} diagnosticsCount={1204} hasBlocking={false} />,
    );
    expect(screen.getByText("99+")).toBeInTheDocument();
  });
});
