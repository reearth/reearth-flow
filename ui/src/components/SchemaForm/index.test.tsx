/**
 * The form, rendered against the engine's real schemas.
 *
 * Where these overlap with `src/lib/schemaForm/reportedBugs.test.ts`, that file
 * pins what the schema compiles to and this one pins what the user sees and can
 * do — the two failures were distinct: a valid config reported invalid, and a
 * form that could not express "unset".
 */
import { readFileSync } from "fs";

import { render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import type { FlowSchema } from "@flow/lib/schemaForm";

import { SchemaForm } from "./index";

const actions = JSON.parse(
  readFileSync("../engine/schema/actions.json", "utf8"),
).actions as { name: string; parameter?: FlowSchema }[];

const schemaFor = (name: string) => {
  const action = actions.find((entry) => entry.name === name);
  if (!action?.parameter) throw new Error(`${name} has no parameter schema`);
  return action.parameter;
};

/**
 * The form is fully controlled — in the app, `ParamsDialog` feeds every change
 * back through Yjs. The harness does the same, so an interaction sees the state
 * its predecessor produced rather than the one the test started with.
 */
const mount = (name: string, formData: unknown) => {
  const onChange = vi.fn();
  const onValidationChange = vi.fn();

  const Harness: React.FC = () => {
    const [data, setData] = useState(formData);
    return (
      <SchemaForm
        schema={schemaFor(name)}
        actionName={name}
        defaultFormData={data}
        onChange={(next, key) => {
          onChange(next, key);
          setData(next);
        }}
        onValidationChange={onValidationChange}
        onFlowExprEditorOpen={() => {}}
      />
    );
  };

  return { onChange, onValidationChange, ...render(<Harness />) };
};

const validity = (mock: ReturnType<typeof vi.fn>) =>
  mock.mock.calls.map((call) => call[0]);

const lastOf = <T,>(values: T[]): T | undefined => values[values.length - 1];

const lastCall = (mock: ReturnType<typeof vi.fn>) => lastOf(mock.mock.calls);

const openDropdown = async (label: string) => {
  await userEvent.click(screen.getByRole("button", { name: label }));
  return screen.findAllByRole("menuitem");
};

describe("mounting a form", () => {
  it("writes nothing back", () => {
    // Opening a dialog is not an edit. The old form fired onChange on mount
    // with a large object of invented defaults, which went straight into Yjs.
    const { onChange } = mount("HTTP Caller", {});
    expect(onChange).not.toHaveBeenCalled();
  });

  it("reports a config the engine accepts as valid", () => {
    const { onValidationChange } = mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
      response: { responseEncoding: null },
    });
    expect(validity(onValidationChange)).toEqual([true]);
  });

  it("reports a config the engine rejects as invalid", () => {
    const { onValidationChange } = mount("HTTP Caller", {});
    expect(validity(onValidationChange)).toEqual([false]);
  });

  it("does not shout about errors the user has not caused yet", () => {
    mount("HTTP Caller", {});
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });
});

describe("optional sections", () => {
  it("leaves them collapsed rather than half-filled", () => {
    mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
    });

    // An optional object offers to be added; an optional union simply has no
    // variant chosen. Neither demands to be filled in.
    expect(
      screen.getByRole("button", { name: "Add Rate Limiting" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Authentication" }),
    ).toHaveTextContent("Not set");
    // The old form rendered Basic Authentication expanded, with empty required
    // username and password fields nobody had asked for.
    expect(screen.queryByText("Username")).not.toBeInTheDocument();
  });

  it("fills only the schema's own defaults when one is added", async () => {
    const { onChange } = mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
    });

    await userEvent.click(
      screen.getByRole("button", { name: "Add Rate Limiting" }),
    );

    const [data] = lastCall(onChange) as [Record<string, unknown>];
    expect(data.rateLimit).toEqual({ intervalMs: 1000, timing: "burst" });
  });

  it("can be removed again", async () => {
    const { onChange } = mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
      timeouts: { connectionTimeout: 30 },
    });

    await userEvent.click(
      screen.getByRole("button", { name: "Remove Timeouts" }),
    );

    const [data] = lastCall(onChange) as [Record<string, unknown>];
    expect(data.timeouts).toBeUndefined();
  });
});

describe("a nullable enum", () => {
  it("offers an explicit way back to unset", async () => {
    mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
      response: { responseEncoding: "text" },
    });

    const items = await openDropdown("Response Encoding");
    expect(items.map((item) => item.textContent)).toEqual([
      "Not set",
      "Text",
      "Base64",
    ]);
  });

  it("produces null, which the engine's schema accepts", async () => {
    const { onChange, onValidationChange } = mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
      response: { responseEncoding: "text" },
    });

    const items = await openDropdown("Response Encoding");
    await userEvent.click(
      items.find((item) => item.textContent === "Not set") as HTMLElement,
    );

    const [data] = lastCall(onChange) as [
      { response: Record<string, unknown> },
    ];
    expect(data.response.responseEncoding).toBeUndefined();
    expect(lastOf(validity(onValidationChange))).toBe(true);
  });

  it("shows unset as unset rather than picking the first option", () => {
    mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
      response: { responseEncoding: null },
    });
    expect(
      screen.getByRole("button", { name: "Response Encoding" }),
    ).toHaveTextContent("Not set");
  });
});

describe("a discriminated union", () => {
  it("renders one control, not a variant picker plus a tag dropdown", () => {
    mount("Feature Writer", {
      format: { type: "csv" },
      output: { type: "string", value: "out.csv" },
    });

    const format = screen.getByRole("button", { name: "Format" });
    expect(format).toHaveTextContent("CSV");
    // The tag was rendered as a second, required "type" dropdown repeating the
    // choice the user had just made above it.
    expect(screen.queryByText("type")).not.toBeInTheDocument();
  });

  it("keeps the tag in the data when the variant changes", async () => {
    const { onChange } = mount("Feature Writer", {
      format: { type: "csv" },
      output: { type: "string", value: "out.csv" },
    });

    const items = await openDropdown("Format");
    await userEvent.click(
      items.find((item) => item.textContent === "JSON") as HTMLElement,
    );

    const [data] = lastCall(onChange) as [{ format: Record<string, unknown> }];
    expect(data.format.type).toBe("json");
  });

  it("carries shared fields across a switch instead of discarding them", async () => {
    // CSV Reader's geometry is untagged: WKT has `column`, coordinates have
    // `xColumn`/`yColumn`. Switching to a variant and back should not lose what
    // was typed.
    const { onChange } = mount("CSV Reader", {
      dataset: { type: "string", value: "in.csv" },
      geometry: { xColumn: "lon", yColumn: "lat" },
    });

    const items = await openDropdown("Geometry Configuration");
    await userEvent.click(
      items.find((item) => item.textContent === "WKT Column") as HTMLElement,
    );
    const afterFirst = lastCall(onChange)?.[0] as {
      geometry: Record<string, unknown>;
    };
    expect(afterFirst.geometry).not.toHaveProperty("xColumn");

    const back = await openDropdown("Geometry Configuration");
    await userEvent.click(
      back.find(
        (item) => item.textContent === "Coordinate Columns",
      ) as HTMLElement,
    );
    const afterSecond = lastCall(onChange)?.[0] as {
      geometry: Record<string, unknown>;
    };
    expect(afterSecond.geometry.xColumn).toBe("lon");
    expect(afterSecond.geometry.yColumn).toBe("lat");
  });
});

describe("editing", () => {
  it("reports the dot path of the field that changed", async () => {
    const { onChange } = mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
      response: { responseBodyAttribute: "_response_body" },
    });

    await userEvent.type(screen.getByLabelText("Response Body Attribute"), "X");

    const [, key] = lastCall(onChange) as [unknown, string];
    expect(key).toBe("response.responseBodyAttribute");
  });

  it("shows errors once the user has engaged", async () => {
    mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
    });
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();

    // Rate limiting requires a request count, which adding the section cannot
    // supply — so the error belongs to the user's action, and is shown.
    await userEvent.click(
      screen.getByRole("button", { name: "Add Rate Limiting" }),
    );

    // Nothing is written under the field: a required field left blank is
    // already marked by its asterisk and by the red border below.
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(screen.getByLabelText("Requests").getAttribute("aria-invalid")).toBe(
      "true",
    );
  });

  it("highlights a blank required expression, which has no text to fall back on", async () => {
    const { container } = mount("Feature Filter", { conditions: [] });

    await userEvent.click(screen.getByRole("button", { name: "Add item" }));

    // The expression chip is not an <input>, so it needs the border applied
    // explicitly — without it a blank required expression showed nothing at all
    // once the message was dropped.
    const chip = container.querySelector(".bg-muted\\/30");
    expect(chip?.className).toContain("border-destructive");
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("marks a required field that is blank without writing under it", async () => {
    mount("Feature Writer", {
      format: { type: "csv" },
      output: { type: "string", value: "out.csv" },
    });

    const output = screen.getByDisplayValue("out.csv");
    await userEvent.clear(output);
    await userEvent.type(output, "x");
    await userEvent.clear(output);

    expect(screen.queryByText(/required/i)).not.toBeInTheDocument();
  });

  it("still writes the messages that say something a highlight cannot", async () => {
    mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
      response: { responseEncoding: "not-an-option" },
    });

    // Touch the form so errors are shown, then check the message survived.
    await userEvent.click(screen.getByRole("button", { name: "Add Timeouts" }));

    const alerts = await screen.findAllByRole("alert");
    expect(alerts.map((alert) => alert.textContent).join(" ")).toMatch(
      /Not one of the allowed values/,
    );
  });
});

describe("array items", () => {
  const filterWith = (conditions: unknown[]) => ({ conditions });

  it("names each row after the list it belongs to", () => {
    mount(
      "Feature Filter",
      filterWith([{ outputPort: "a" }, { outputPort: "b" }]),
    );

    // Not the raw property name ("conditions") repeated on every row.
    expect(screen.getByText("Filter Conditions-1")).toBeInTheDocument();
    expect(screen.getByText("Filter Conditions-2")).toBeInTheDocument();
    expect(screen.queryByText("conditions")).not.toBeInTheDocument();
  });

  it("adds an object row as an empty object, not a hole", async () => {
    // Seeding an object with no defaults of its own from `undefined` returned
    // `undefined`, which landed in the array and read back as "must be object".
    const { onChange } = mount("Feature Filter", filterWith([]));

    await userEvent.click(screen.getByRole("button", { name: "Add item" }));

    const [data] = lastCall(onChange) as [{ conditions: unknown[] }];
    expect(data.conditions).toHaveLength(1);
    expect(data.conditions[0]).toEqual({});
    expect(screen.queryByText(/must be object/i)).not.toBeInTheDocument();
  });

  it("keeps each row's controls beside its heading", () => {
    mount("Feature Filter", filterWith([{ outputPort: "a" }]));
    expect(
      screen.getByRole("button", { name: "Move item up" }),
    ).toBeInTheDocument();
    expect(
      screen.getByRole("button", { name: "Remove item" }),
    ).toBeInTheDocument();
  });

  it("reorders rows without losing their contents", async () => {
    const { onChange } = mount(
      "Feature Filter",
      filterWith([{ outputPort: "first" }, { outputPort: "second" }]),
    );

    const [moveFirstDown] = screen.getAllByRole("button", {
      name: "Move item down",
    });
    await userEvent.click(moveFirstDown);

    const [data] = lastCall(onChange) as [
      { conditions: { outputPort: string }[] },
    ];
    expect(data.conditions.map((row) => row.outputPort)).toEqual([
      "second",
      "first",
    ]);
  });
});

describe("density", () => {
  it("does not repeat a field's description under every input", () => {
    // Sections keep their descriptions; leaves do not, or a list of conditions
    // is three times taller than it needs to be.
    mount("Feature Filter", { conditions: [{ outputPort: "a" }] });

    expect(
      screen.getByText(
        /List of conditions and their corresponding output ports/,
      ),
    ).toBeInTheDocument();
    expect(
      screen.queryByText(/Boolean expression evaluated against each feature/),
    ).not.toBeInTheDocument();
  });

  it("labels the expression controls for a screen reader", () => {
    mount("Feature Filter", { conditions: [{ outputPort: "a" }] });
    expect(
      screen.getByRole("button", { name: "Open FlowExpr Editor" }),
    ).toBeInTheDocument();
  });
});

describe("every action", () => {
  it("renders without throwing", () => {
    const failures: string[] = [];
    for (const action of actions) {
      if (!action.parameter) continue;
      try {
        const view = render(
          <SchemaForm
            schema={action.parameter}
            actionName={action.name}
            defaultFormData={{}}
            onChange={() => {}}
          />,
        );
        view.unmount();
      } catch (error) {
        failures.push(`${action.name}: ${(error as Error).message}`);
      }
    }
    expect(failures).toEqual([]);
  });
});
