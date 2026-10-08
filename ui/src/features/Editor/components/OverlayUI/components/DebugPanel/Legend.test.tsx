import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, test } from "vitest";

import Legend from "./Legend";

describe("Legend", () => {
  test("shows a single entry as it is", () => {
    render(<Legend entries={[{ color: "#f00", label: "Back of a face" }]} />);

    expect(screen.getByText("Back of a face")).toBeInTheDocument();
    expect(screen.queryByRole("button")).not.toBeInTheDocument();
  });

  test("folds several entries behind a button, and opens and closes", () => {
    render(
      <Legend
        entries={[
          { color: "#00f", label: "Feature" },
          { color: "#fa0", label: "Selected" },
        ]}
      />,
    );

    expect(screen.queryByText("Feature")).not.toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Show legend" }));
    expect(screen.getByText("Feature")).toBeInTheDocument();
    expect(screen.getByText("Selected")).toBeInTheDocument();

    fireEvent.click(screen.getByRole("button", { name: "Hide legend" }));
    expect(screen.queryByText("Feature")).not.toBeInTheDocument();
  });

  test("shows nothing without entries", () => {
    const { container } = render(<Legend entries={[]} />);
    expect(container).toBeEmptyDOMElement();
  });
});
