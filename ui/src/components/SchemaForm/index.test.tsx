/**
 * The form, rendered against the engine's real schemas.
 *
 * Where these overlap with `src/lib/schemaForm/reportedBugs.test.ts`, that file
 * pins what the schema compiles to and this one pins what the user sees and can
 * do — the two failures were distinct: a valid config reported invalid, and a
 * form that could not express "unset".
 */
import { readFileSync } from "fs";

import { act, render, screen } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { useState } from "react";
import { describe, expect, it, vi } from "vitest";

import { isValid, validate, type FlowSchema } from "@flow/lib/schemaForm";

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

/**
 * The same harness, plus a way to put a value in from outside the form — what
 * a collaborator's edit arriving over Yjs looks like from the form's side.
 */
const mountShared = (name: string, formData: unknown) => {
  const onChange = vi.fn();
  let fromElsewhere: (next: unknown) => void = () => {};

  const Harness: React.FC = () => {
    const [data, setData] = useState(formData);
    fromElsewhere = setData;
    return (
      <SchemaForm
        schema={schemaFor(name)}
        actionName={name}
        defaultFormData={data}
        onChange={(next, key) => {
          onChange(next, key);
          setData(next);
        }}
        onFlowExprEditorOpen={() => {}}
      />
    );
  };

  const rendered = render(<Harness />);
  return {
    onChange,
    collaboratorWrites: (next: unknown) => act(() => fromElsewhere(next)),
    ...rendered,
  };
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

  it("leaves a required field the user has not reached alone", async () => {
    const { onValidationChange } = mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
    });

    await userEvent.click(
      screen.getByRole("button", { name: "Add Rate Limiting" }),
    );

    // Rate limiting requires a request count. The asterisk beside the label
    // says so; nothing is written underneath and the input is not coloured in.
    const requests = screen.getByLabelText("Requests");
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
    expect(requests.getAttribute("aria-invalid")).toBe("false");
    expect(requests.className).not.toContain("border-destructive");

    // It still counts against the form, which is what gates Update.
    expect(lastOf(validity(onValidationChange))).toBe(false);
  });

  it("colours in a field whose value is actually wrong", async () => {
    mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
      response: { responseEncoding: "not-an-option" },
    });

    await userEvent.click(screen.getByRole("button", { name: "Add Timeouts" }));

    // Here the colour points at something the field cannot say for itself.
    const encoding = screen.getByRole("button", { name: "Response Encoding" });
    expect(encoding.className).toContain("border-destructive");
    expect(await screen.findAllByRole("alert")).not.toHaveLength(0);
  });

  it("leaves a blank required expression uncoloured too", async () => {
    const { container } = mount("Feature Filter", { conditions: [] });

    await userEvent.click(screen.getByRole("button", { name: "Add item" }));

    const chip = container.querySelector(".bg-muted\\/30");
    expect(chip?.className).not.toContain("border-destructive");
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

describe("a required union with no null branch", () => {
  // Coordinate Frame Reprojector's destinationFrame: required, `oneOf` of a CRS
  // object and the bare string "euclidean", and no null branch at all. There is
  // no such thing as unset here — only unchosen.
  const mountFrame = () => mountShared("Coordinate Frame Reprojector", {});

  it("prompts for a choice instead of claiming to be unset", async () => {
    mountFrame();
    expect(
      screen.getByRole("button", { name: "Destination Frame" }),
    ).toHaveTextContent("Select...");

    const items = await openDropdown("Destination Frame");
    // No way to go back to nothing, because the schema does not allow it.
    expect(items.map((item) => item.textContent)).toEqual(["CRS", "Euclidean"]);
  });

  it("keeps an untagged variant selected before its fields are filled", async () => {
    mountFrame();

    const items = await openDropdown("Destination Frame");
    await userEvent.click(
      items.find((item) => item.textContent === "CRS") as HTMLElement,
    );

    // CRS is recognised by the `crs` field it carries, so the moment it is
    // chosen the value matches nothing. The choice has to be remembered, or the
    // control falls straight back to showing nothing selected.
    expect(
      screen.getByRole("button", { name: "Destination Frame" }),
    ).toHaveTextContent("CRS");
    expect(screen.getByText("crs")).toBeInTheDocument();
  });

  it("says nothing about the branch the user did not choose", async () => {
    mountFrame();

    const items = await openDropdown("Destination Frame");
    await userEvent.click(
      items.find((item) => item.textContent === "CRS") as HTMLElement,
    );

    // The Euclidean branch reports "must be string" for a CRS object. That is
    // AJV describing a shape the user did not pick, not a problem with theirs.
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("switches to the variant that is a bare constant", async () => {
    const { onChange } = mountFrame();

    const items = await openDropdown("Destination Frame");
    await userEvent.click(
      items.find((item) => item.textContent === "Euclidean") as HTMLElement,
    );

    const [data] = lastCall(onChange) as [Record<string, unknown>];
    expect(data.destinationFrame).toBe("euclidean");
    expect(
      screen.getByRole("button", { name: "Destination Frame" }),
    ).toHaveTextContent("Euclidean");
  });

  it("lets go of the choice when the value is cleared elsewhere", async () => {
    // The choice lives in component state only while the value cannot express
    // it. That made it outlive the data: once a collaborator cleared the field
    // the dropdown went on naming a variant the params no longer held, with
    // the section beneath it open over nothing.
    const { collaboratorWrites } = mountFrame();

    const items = await openDropdown("Destination Frame");
    await userEvent.click(
      items.find((item) => item.textContent === "CRS") as HTMLElement,
    );
    expect(
      screen.getByRole("button", { name: "Destination Frame" }),
    ).toHaveTextContent("CRS");

    collaboratorWrites({});

    expect(
      screen.getByRole("button", { name: "Destination Frame" }),
    ).toHaveTextContent("Select...");
    expect(screen.queryByText("crs")).not.toBeInTheDocument();
  });

  it("keeps the choice when the value it wrote comes back unchanged", async () => {
    // The mirror image, and the reason the check is against the value this
    // control wrote rather than against "matches nothing": an untagged variant
    // matches nothing until its fields are filled, so a round trip through Yjs
    // that returns the same value must not read as someone else's clear.
    const { onChange, collaboratorWrites } = mountFrame();

    const items = await openDropdown("Destination Frame");
    await userEvent.click(
      items.find((item) => item.textContent === "CRS") as HTMLElement,
    );

    const [echoed] = lastCall(onChange) as [unknown];
    collaboratorWrites(echoed);

    expect(
      screen.getByRole("button", { name: "Destination Frame" }),
    ).toHaveTextContent("CRS");
  });

  it("follows the value when it identifies a different variant", () => {
    mount("Coordinate Frame Reprojector", { destinationFrame: "euclidean" });
    expect(
      screen.getByRole("button", { name: "Destination Frame" }),
    ).toHaveTextContent("Euclidean");
  });

  it("puts the choice where another editor can see it", async () => {
    // The choice has to be in the data, not in component state: a collaborator
    // renders from the value alone. Picking CRS used to write `{}`, which
    // identified no variant, so everyone else went on seeing "Select..." until
    // the first EPSG code was typed.
    const { onChange } = mountFrame();

    const items = await openDropdown("Destination Frame");
    await userEvent.click(
      items.find((item) => item.textContent === "CRS") as HTMLElement,
    );

    const [data] = lastCall(onChange) as [Record<string, unknown>];
    expect(data.destinationFrame).toEqual({ crs: null });
  });

  it("shows that choice on a client that only received the value", () => {
    // A second editor, rendering from the value with no state of its own.
    render(
      <SchemaForm
        schema={schemaFor("Coordinate Frame Reprojector")}
        actionName="Coordinate Frame Reprojector"
        defaultFormData={{ destinationFrame: { crs: null } }}
        onChange={() => {}}
      />,
    );

    expect(
      screen.getByRole("button", { name: "Destination Frame" }),
    ).toHaveTextContent("CRS");
    expect(screen.getByText("crs")).toBeInTheDocument();
    // Chosen but unfilled is not an error to shout about — and not savable.
    expect(screen.queryByRole("alert")).not.toBeInTheDocument();
  });

  it("stays unsavable until the chosen variant is filled in", () => {
    const schema = schemaFor("Coordinate Frame Reprojector");
    expect(isValid(validate(schema, { destinationFrame: { crs: null } }))).toBe(
      false,
    );
    expect(
      isValid(
        validate(schema, {
          destinationFrame: { crs: { type: "flowExpr", value: "EPSG:4326" } },
        }),
      ),
    ).toBe(true);
  });

  it("writes nothing for a variant told apart by its own type", async () => {
    // Attribute Range Mapper's default value is Text | Number | True-or-False.
    // Seeding an object into a string variant wrote `{}` — an object where the
    // schema wants a string, and identical for all three variants.
    const { onChange } = mount("Attribute Range Mapper", {});

    const items = await openDropdown("Default Value");
    await userEvent.click(
      items.find((item) => item.textContent === "Number") as HTMLElement,
    );

    const [data] = lastCall(onChange) as [Record<string, unknown>];
    expect(data).not.toHaveProperty("defaultValue");
    expect(
      screen.getByRole("button", { name: "Default Value" }),
    ).toHaveTextContent("Number");
  });

  it("still reports a value inside the chosen branch that is wrong", () => {
    const { onValidationChange } = mount("Coordinate Frame Reprojector", {
      destinationFrame: { crs: "not a code object" },
    });
    // Suppressing the other branch's complaints must not suppress this one.
    expect(lastOf(validity(onValidationChange))).toBe(false);
  });
});

describe("clearing a field", () => {
  it("removes the key instead of leaving it present and undefined", async () => {
    const { onChange } = mount("HTTP Caller", {
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
    // A key present with an undefined value is still a key: `Object.keys`
    // reports it, so an object with `additionalProperties: false` rejects it,
    // and it rides into the saved params as a phantom entry.
    expect("responseEncoding" in data.response).toBe(false);
    expect(Object.keys(data.response)).not.toContain("responseEncoding");
  });

  it("removes a whole optional section by key, not by blanking it", async () => {
    const { onChange } = mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
      timeouts: { connectionTimeout: 30 },
    });

    await userEvent.click(
      screen.getByRole("button", { name: "Remove Timeouts" }),
    );

    const [data] = lastCall(onChange) as [Record<string, unknown>];
    expect("timeouts" in data).toBe(false);
  });

  it("stays valid, since the schema permits the field to be unset", async () => {
    const { onValidationChange } = mount("HTTP Caller", {
      url: { type: "string", value: "https://example.test" },
      response: { responseEncoding: "text" },
    });

    const items = await openDropdown("Response Encoding");
    await userEvent.click(
      items.find((item) => item.textContent === "Not set") as HTMLElement,
    );

    expect(lastOf(validity(onValidationChange))).toBe(true);
  });
});

describe("a root the schema allows to be null", () => {
  const nullableRoot = {
    anyOf: [
      {
        type: "object",
        required: ["a"],
        properties: { a: { type: "string", title: "A" } },
      },
      { type: "null" },
    ],
  } as never;

  it("is reported valid when the value is null", () => {
    const onValidationChange = vi.fn();
    render(
      <SchemaForm
        schema={nullableRoot}
        defaultFormData={null}
        onChange={() => {}}
        onValidationChange={onValidationChange}
      />,
    );
    // `seeded ?? {}` turned the null into `{}` before AJV saw it, so a form
    // whose data the schema accepts was reported invalid.
    expect(validity(onValidationChange)).toEqual([true]);
  });

  it("still treats an absent root as an empty object", () => {
    const onValidationChange = vi.fn();
    render(
      <SchemaForm
        schema={nullableRoot}
        defaultFormData={undefined}
        onChange={() => {}}
        onValidationChange={onValidationChange}
      />,
    );
    expect(validity(onValidationChange)).toEqual([false]);
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

describe("a union of bare scalars", () => {
  /**
   * A variant recognised by the type of the value itself has no keys to write,
   * so choosing one leaves the value empty and matching nothing — the window
   * the local choice exists to cover. No action publishes this shape today;
   * `UnionField` handles it, so it is pinned against a schema written here.
   */
  const scalarUnion: FlowSchema = {
    type: "object",
    properties: {
      limit: {
        title: "Limit",
        anyOf: [
          { title: "Text", type: "string" },
          { title: "Number", type: "number" },
        ],
      },
    },
  };

  const mountScalars = () => {
    const Harness: React.FC = () => {
      const [data, setData] = useState<unknown>({});
      return (
        <SchemaForm
          schema={scalarUnion}
          defaultFormData={data}
          onChange={setData}
        />
      );
    };
    return render(<Harness />);
  };

  it("holds the choice through the window where it writes nothing", async () => {
    mountScalars();

    const items = await openDropdown("Limit");
    await userEvent.click(
      items.find((item) => item.textContent === "Number") as HTMLElement,
    );

    // Writing `{}` here would store an object where the schema wants a number
    // and make Text and Number indistinguishable, so the choice is held in the
    // control until the user types — one keystroke wide, but it has to hold.
    expect(screen.getByRole("button", { name: "Limit" })).toHaveTextContent(
      "Number",
    );
  });
});
