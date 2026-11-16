import { render, screen } from "@testing-library/react";
import { describe, expect, test } from "vitest";

import { RequiredVariablesNotice } from "./RequiredVariablesNotice";

describe("RequiredVariablesNotice", () => {
  test("reminds that each CMS run must provide the missing variables", () => {
    render(
      <RequiredVariablesNotice missing={["cityGmlPath"]} eventSource="CMS" />,
    );

    expect(
      screen.getByText("Each CMS run must provide: cityGmlPath"),
    ).toHaveClass("text-warning");
  });

  test("says a scheduled trigger needs the values", () => {
    render(
      <RequiredVariablesNotice
        missing={["cityGmlPath"]}
        eventSource="TIME_DRIVEN"
      />,
    );

    expect(
      screen.getByText("Required variables need a value: cityGmlPath"),
    ).toHaveClass("text-destructive");
  });

  test("shows nothing when no required variable is missing", () => {
    const { container } = render(
      <RequiredVariablesNotice missing={[]} eventSource="CMS" />,
    );

    expect(container).toBeEmptyDOMElement();
  });
});
