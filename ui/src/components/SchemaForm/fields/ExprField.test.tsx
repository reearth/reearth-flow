import { readFileSync } from "fs";

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import type { FlowSchema } from "@flow/lib/schemaForm";

import { SchemaForm } from "../index";

const ACTION = "GeoJSON Reader";
const schema = (
  JSON.parse(readFileSync("../engine/schema/actions.json", "utf8")).actions as {
    name: string;
    parameter: FlowSchema;
  }[]
).find((entry) => entry.name === ACTION)?.parameter as FlowSchema;

const mount = () => {
  const onChange = vi.fn();
  const Harness: React.FC = () => {
    const [data, setData] = useState<unknown>({});
    return (
      <>
        <SchemaForm
          schema={schema}
          actionName={ACTION}
          defaultFormData={data}
          onChange={(next) => {
            onChange(next);
            setData(next);
          }}
          onFlowExprEditorOpen={() => {}}
        />
        <button>elsewhere</button>
      </>
    );
  };
  render(<Harness />);
  return onChange;
};

const lastDataset = (onChange: ReturnType<typeof vi.fn>) =>
  (onChange.mock.lastCall?.[0] as { dataset?: unknown }).dataset;

describe("ExprField inline editing", () => {
  it("keeps the input editable while a variable reference is typed", async () => {
    const onChange = mount();
    const expression = 'variables["hello"] + "/a.csv"';

    await userEvent.type(
      screen.getByLabelText("File Path"),
      expression.replace(/[[{]/g, (c) => c + c),
    );

    expect(screen.getByLabelText("File Path")).toHaveValue(expression);
    expect(lastDataset(onChange)).toEqual({
      type: "string",
      value: expression,
    });
  });

  it("saves a bare variable reference as an expression once editing finishes", async () => {
    const onChange = mount();
    const expression = 'variables["hello"]';

    await userEvent.type(
      screen.getByLabelText("File Path"),
      expression.replace(/[[{]/g, (c) => c + c),
    );
    expect(screen.getByLabelText("File Path")).toHaveValue(expression);
    expect(lastDataset(onChange)).toEqual({
      type: "string",
      value: expression,
    });

    await userEvent.click(screen.getByText("elsewhere"));

    expect(lastDataset(onChange)).toEqual({
      type: "flowExpr",
      value: expression,
    });
    expect(screen.queryByLabelText("File Path")).not.toBeInTheDocument();
  });
});
