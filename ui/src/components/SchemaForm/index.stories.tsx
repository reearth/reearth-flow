/**
 * The action parameter form, one story per thing it has to draw.
 *
 * The form is fully controlled — in the app `ParamsDialog` feeds every change
 * back through Yjs, so a story that dropped `onChange` on the floor would
 * render a form nothing can type into. `Harness` does what the dialog does:
 * holds the data, feeds it back, and shows what the form emitted. A story is
 * then a schema plus the state it opens in.
 *
 * `index.test.tsx` pins behaviour against the engine's published schemas; these
 * stories are for looking at it. The schemas below are hand-written in the
 * shape `schemars` emits (`engine/schema/actions.json`) so each field kind can
 * be seen in isolation — `EngineAction` renders the real ones off a local
 * engine.
 */
import {
  CaretDownIcon,
  CaretLeftIcon,
  CaretRightIcon,
  CopyIcon,
} from "@phosphor-icons/react";
import type { Meta, StoryObj } from "@storybook/react-vite";
import type { JSONSchema7 } from "json-schema";
import { useCallback, useEffect, useMemo, useState } from "react";
import { I18nextProvider } from "react-i18next";

import i18n from "@flow/lib/i18n/i18n";
import type { AwarenessUser } from "@flow/types";

import { Button } from "../buttons";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "../DropdownMenu";
import { TooltipProvider } from "../Tooltip";

import type { EditorContext } from "./context";
import {
  CONTAINERS_SCHEMA,
  FIELD_KINDS_SCHEMA,
  NULLABILITY_SCHEMA,
  PYTHON_SCRIPT_SCHEMA,
  UNIONS_SCHEMA,
  UNSUPPORTED_SCHEMA,
  VALIDATION_SCHEMA,
} from "./storyFixtures";

import { SchemaForm } from ".";

const meta = {
  component: SchemaForm,
  decorators: [
    (Story) => (
      // `i18n` is initialised by importing it; the app's own provider reads the
      // user's language over GraphQL, which a story has no business doing.
      <I18nextProvider i18n={i18n}>
        <TooltipProvider>
          <Story />
        </TooltipProvider>
      </I18nextProvider>
    ),
  ],
  parameters: { layout: "padded" },
  tags: ["autodocs"],
  args: { onChange: () => {} },
  argTypes: {},
} satisfies Meta<typeof SchemaForm>;

export default meta;
type Story = StoryObj<typeof meta>;

// ---------------------------------------------------------------------------
// Harness
// ---------------------------------------------------------------------------

type HarnessProps = {
  schema?: JSONSchema7;
  actionName?: string;
  initialData?: unknown;
  readonly?: boolean;
  fieldFocusMap?: Record<string, AwarenessUser[]>;
  /** What to watch for in this story. */
  note?: string;
  /** Extra controls above the form (schema pickers and the like). */
  controls?: React.ReactNode;
};

/**
 * The form plus an inspector: validity, the last field to change, who the form
 * thinks is looking at what, and the data as it would reach Yjs. Most of what
 * these stories are for is invisible in the form itself — a cleared field
 * deleting its key, errors held back until the user engages — so the right
 * panel is half the story.
 */
const Harness: React.FC<HarnessProps> = ({
  schema,
  actionName,
  initialData,
  readonly,
  fieldFocusMap,
  note,
  controls,
}) => {
  const [data, setData] = useState<unknown>(initialData);
  const [valid, setValid] = useState(true);
  const [changedKey, setChangedKey] = useState<string>();
  const [focusedKey, setFocusedKey] = useState<string | null>(null);
  const [editorContext, setEditorContext] = useState<EditorContext>();

  // A story switching schemas (EngineAction, CustomSchema) starts that schema's
  // data from scratch, exactly as opening a different node does.
  useEffect(() => setData(initialData), [schema, initialData]);

  const handleChange = useCallback((next: unknown, key?: string) => {
    setData(next);
    setChangedKey(key);
  }, []);

  const handleEditorOpen = useCallback(
    (context: EditorContext) => setEditorContext(context),
    [],
  );

  return (
    <div className="flex h-[80vh] w-full gap-2">
      <div className="flex min-w-0 flex-1 flex-col gap-2">
        {controls}
        <div className="flex min-h-0 flex-1 flex-col rounded border p-3">
          <SchemaForm
            schema={schema}
            actionName={actionName}
            defaultFormData={data}
            readonly={readonly}
            fieldFocusMap={fieldFocusMap}
            onFieldFocus={setFocusedKey}
            onChange={handleChange}
            onValidationChange={setValid}
            onEditorOpen={handleEditorOpen}
            onPythonEditorOpen={handleEditorOpen}
            onFlowExprEditorOpen={handleEditorOpen}
          />
        </div>
      </div>
      <div className="flex w-[380px] shrink-0 flex-col gap-2 overflow-auto">
        {note && (
          <p className="rounded border bg-card p-2 text-xs text-muted-foreground">
            {note}
          </p>
        )}
        <dl className="grid grid-cols-[auto_1fr] gap-x-3 gap-y-1 rounded border bg-card p-2 text-xs">
          <dt className="text-muted-foreground">Valid</dt>
          <dd className={valid ? "text-success" : "text-destructive"}>
            {valid ? "yes" : "no"}
          </dd>
          <dt className="text-muted-foreground">Changed field</dt>
          <dd className="truncate font-mono">{changedKey ?? "—"}</dd>
          <dt className="text-muted-foreground">Focused field</dt>
          <dd className="truncate font-mono">{focusedKey ?? "—"}</dd>
        </dl>
        {editorContext && (
          <div className="rounded border bg-card p-2 text-xs">
            <p className="mb-1 text-muted-foreground">
              Editor requested for{" "}
              <span className="font-mono">{editorContext.key}</span> — the app
              opens the FlowExpr or Python dialog here.
            </p>
            <pre className="overflow-auto">
              {JSON.stringify(editorContext.value, null, 2)}
            </pre>
          </div>
        )}
        <div className="relative min-h-0 flex-1 rounded border bg-card p-2">
          <p className="mb-1 text-xs text-muted-foreground">Form data</p>
          <Button
            size="sm"
            variant="outline"
            className="absolute top-1 right-1"
            onClick={() =>
              navigator.clipboard.writeText(JSON.stringify(data, null, 2))
            }>
            <CopyIcon />
          </Button>
          <pre className="h-full overflow-auto text-xs">
            {JSON.stringify(data, null, 2) ?? "undefined"}
          </pre>
        </div>
      </div>
    </div>
  );
};

// ---------------------------------------------------------------------------
// Every field kind
// ---------------------------------------------------------------------------

export const AllFieldKinds: Story = {
  render: () => (
    <Harness
      schema={FIELD_KINDS_SCHEMA}
      note="Every kind the renderer knows: string, number, boolean, enum, array, object, map, union, expr, color, wysiwyg, unsupported. Nothing is written to the form data until you edit something — the defaults you can see are display-only, because mounting a dialog is not an edit."
    />
  ),
};

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

export const Validation: Story = {
  render: () => (
    <Harness
      schema={VALIDATION_SCHEMA}
      note="Opens invalid but quiet: required fields carry an asterisk and nothing else until you edit something, so a half-filled action does not greet the user with red they did not cause. Watch Valid in the panel — that is what gates the dialog's Update button."
    />
  ),
};

// ---------------------------------------------------------------------------
// Nullability
// ---------------------------------------------------------------------------

export const Nullability: Story = {
  render: () => (
    <Harness
      schema={NULLABILITY_SCHEMA}
      initialData={{
        attribute: "bldg:class",
        limit: 50,
        caseSensitive: true,
        bounds: { minX: 0, maxX: 100 },
        tags: ["plateau"],
      }}
      note="Clear a field and its key leaves the form data entirely — it is not set to undefined, which would leave the key present for an object with additionalProperties: false to reject and ride into the saved params as a phantom entry. Watch keys disappear from the panel."
    />
  ),
};

// ---------------------------------------------------------------------------
// Unions
// ---------------------------------------------------------------------------

export const Unions: Story = {
  render: () => (
    <Harness
      schema={UNIONS_SCHEMA}
      initialData={{ projection: { type: "epsg", code: 6668 } }}
      note="A tagged union renders one dropdown, not two: the tag is implied by the variant you pick and is written into the data for you. Switching variants keeps what the new branch can still hold and drops what it cannot."
    />
  ),
};

// ---------------------------------------------------------------------------
// Containers
// ---------------------------------------------------------------------------

export const Containers: Story = {
  render: () => (
    <Harness
      schema={CONTAINERS_SCHEMA}
      initialData={{
        rules: [
          { attribute: "bldg:class", operator: "equals", value: "3001" },
          { attribute: "gml:name", operator: "contains" },
        ],
        tables: {
          "bldg:Building": { columns: ["gml:id", "bldg:class"] },
          "bldg.part": { columns: ["gml:id"], hasHeader: true },
        },
      }}
      note="Changed field in the panel shows the dot path each edit reports — the same key used for awareness focus, error lookup and the Yjs draft patches. A map key's dot is escaped as ~1 so bldg.part does not read back as two segments."
    />
  ),
};

// ---------------------------------------------------------------------------
// Expressions
// ---------------------------------------------------------------------------

/**
 * Copied from the engine's `Python Script Processor`, which is what makes the
 * `script` field open the Python editor rather than the FlowExpr one — the
 * flavour comes from an override keyed by action name and path, not from the
 * schema.
 */
export const Expressions: Story = {
  render: () => (
    <Harness
      schema={PYTHON_SCRIPT_SCHEMA}
      actionName="Python Script Processor"
      initialData={{
        script: { type: "string", value: "print(feature['gml:id'])" },
        pythonFile: { type: "flowExpr", value: "env.get('SCRIPT_URL')" },
      }}
      note="An expr field is either a literal you type inline or an expression chip. The pencil asks the app for an editor; the panel shows the request the dialog would act on. Under this action name `script` is routed to the Python editor and `pythonFile` to FlowExpr."
    />
  ),
};

// ---------------------------------------------------------------------------
// Read-only and collaborative focus
// ---------------------------------------------------------------------------

export const ReadOnly: Story = {
  render: () => (
    <Harness
      readonly
      schema={FIELD_KINDS_SCHEMA}
      initialData={{
        name: "Overlayer",
        featureCount: 200,
        mode: "accurate",
        groupBy: ["bldg:class"],
        renames: { "bldg:class": "class" },
        projection: { type: "epsg", code: 6668 },
        filter: { type: "flowExpr", value: "feature.area > 100" },
        strokeColor: "#4f46e5",
      }}
      note="What a viewer without edit rights sees: every control inert, nothing removable, no editor dialogs."
    />
  ),
};

const OTHER_USERS: Record<string, AwarenessUser[]> = {
  name: [{ clientId: 1, color: "#f59e0b", userName: "Aoi" }],
  "extent.minX": [{ clientId: 2, color: "#10b981", userName: "Ben" }],
  "renames.bldg~1part": [{ clientId: 3, color: "#ef4444", userName: "Chika" }],
};

export const CollaborativeFocus: Story = {
  render: () => (
    <Harness
      schema={FIELD_KINDS_SCHEMA}
      initialData={{
        name: "Overlayer",
        extent: { minX: 0, minY: 0 },
        renames: { "bldg.part": "part" },
      }}
      fieldFocusMap={OTHER_USERS}
      note="fieldFocusMap is keyed by the same dot path, so three fields here are outlined in another user's colour — including a map key whose dot is escaped (renames.bldg~1part). Focus a field yourself and the panel shows the key the app broadcasts back over awareness."
    />
  ),
};

// ---------------------------------------------------------------------------
// Unsupported shapes
// ---------------------------------------------------------------------------

export const Unsupported: Story = {
  render: () => (
    <Harness
      schema={UNSUPPORTED_SCHEMA}
      initialData={{
        mixedType: 42,
        looseArray: [1, "two", { three: true }],
        anythingGoes: { arbitrary: "payload" },
      }}
      note="Edit the JSON and the value survives round-tripping. The reason each field fell through is printed with it."
    />
  ),
};

// ---------------------------------------------------------------------------
// The engine's own schemas
// ---------------------------------------------------------------------------

const fetcher = async (url: string) => {
  const response = await fetch(url);
  if (!response.ok) throw new Error(`${response.status} from ${url}`);
  return await response.json();
};

const ENGINE_URL = "http://localhost:8080";

/**
 * Every published action, walked one at a time.
 *
 * Nothing under `src/` may import from `engine/` — the UI image builds with
 * `context: ui` — so the schemas are fetched from a locally running engine
 * rather than read off disk. `index.test.tsx` is where they are checked in CI.
 */
const EngineActionBrowser: React.FC = () => {
  const [actions, setActions] = useState<{ name: string }[]>([]);
  const [error, setError] = useState<string>();
  const [selected, setSelected] = useState<string>();
  const [schema, setSchema] = useState<JSONSchema7>();
  const [showSchema, setShowSchema] = useState(false);

  useEffect(() => {
    (async () => {
      try {
        const data = await fetcher(`${ENGINE_URL}/actions`);
        setActions(data);
        setSelected(data[0]?.name);
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e));
      }
    })();
  }, []);

  useEffect(() => {
    if (!selected) return;
    let stale = false;
    (async () => {
      try {
        const { parameter } = await fetcher(
          `${ENGINE_URL}/actions/${encodeURIComponent(selected)}`,
        );
        if (!stale) setSchema(parameter);
      } catch (e) {
        if (!stale) {
          setError(e instanceof Error ? e.message : String(e));
        }
      }
    })();
    return () => {
      stale = true;
    };
  }, [selected]);

  const index = useMemo(
    () => actions.findIndex((action) => action.name === selected),
    [actions, selected],
  );

  if (error) {
    return (
      <p className="rounded border p-3 text-sm text-destructive">
        Could not reach the action API at {ENGINE_URL}: {error}. Start it with{" "}
        <span className="font-mono">make run-app</span> from{" "}
        <span className="font-mono">server/api/</span>, or use the Custom Schema
        story instead.
      </p>
    );
  }

  return (
    <Harness
      schema={schema}
      actionName={selected}
      note="The published schemas, rendered by the form the way the params dialog will. Any action drawing an Unsupported field here is a compiler gap, not a schema problem — teach the compiler the shape."
      controls={
        <>
          <div className="flex items-center justify-between gap-2 rounded border p-2">
            <div className="truncate text-sm">
              {selected ?? "Loading actions…"}
              <span className="ml-2 text-xs text-muted-foreground">
                {actions.length > 0 && `${index + 1} / ${actions.length}`}
              </span>
            </div>
            <div className="flex shrink-0 items-center gap-2">
              <Button
                size="sm"
                variant="outline"
                disabled={index <= 0}
                onClick={() => setSelected(actions[index - 1]?.name)}>
                <CaretLeftIcon />
              </Button>
              <Button
                size="sm"
                variant="outline"
                disabled={index < 0 || index >= actions.length - 1}
                onClick={() => setSelected(actions[index + 1]?.name)}>
                <CaretRightIcon />
              </Button>
              <DropdownMenu modal={true}>
                <DropdownMenuTrigger className="flex h-8 items-center rounded border bg-background px-1 hover:bg-accent">
                  <p className="text-sm">Select action</p>
                  <CaretDownIcon />
                </DropdownMenuTrigger>
                <DropdownMenuContent
                  className="h-96 overflow-auto"
                  align="center">
                  {actions.map(({ name }) => (
                    <DropdownMenuItem
                      key={name}
                      onClick={() => setSelected(name)}>
                      {name}
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>
              <Button
                size="sm"
                variant="outline"
                onClick={() => setShowSchema(!showSchema)}>
                {showSchema ? "Hide" : "Show"} schema
              </Button>
            </div>
          </div>
          {showSchema && (
            <pre className="max-h-64 overflow-auto rounded border bg-card p-2 text-xs">
              {JSON.stringify(schema, null, 2)}
            </pre>
          )}
        </>
      }
    />
  );
};

export const EngineAction: Story = {
  render: () => <EngineActionBrowser />,
};

// ---------------------------------------------------------------------------
// Bring your own schema
// ---------------------------------------------------------------------------

/**
 * Paste a schema and see it drawn — the fastest way to reproduce a report about
 * one action, and the only way to look at a custom action's schema without an
 * engine running.
 */
const CustomSchemaEditor: React.FC = () => {
  const [text, setText] = useState(JSON.stringify(FIELD_KINDS_SCHEMA, null, 2));

  const parsed = useMemo<{
    schema?: JSONSchema7;
    name?: string;
    error?: string;
  }>(() => {
    try {
      const value = JSON.parse(text);
      // An action object from actions.json pasted whole is the common case.
      if (value === null || typeof value !== "object" || Array.isArray(value)) {
        return { error: "Expected a schema object or action object" };
      }
      return {
        schema: Object.prototype.hasOwnProperty.call(value, "parameter")
          ? (value.parameter as JSONSchema7)
          : (value as JSONSchema7),
        name: typeof value.name === "string" ? value.name : undefined,
      };
    } catch (e) {
      return { error: e instanceof Error ? e.message : String(e) };
    }
  }, [text]);

  return (
    <Harness
      schema={parsed.schema}
      actionName={parsed.name}
      note="Accepts either a bare parameter schema or a whole action object from engine/schema/actions.json — the action name matters, since the Python editor override is keyed by it."
      controls={
        <div className="flex flex-col gap-1">
          <textarea
            value={text}
            spellCheck={false}
            onChange={(event) => setText(event.target.value)}
            className="h-48 w-full rounded border bg-transparent p-2 font-mono text-xs"
            placeholder="Paste a parameter schema, or an action object from engine/schema/actions.json…"
          />
          {parsed.error && (
            <p className="text-xs text-destructive">
              Invalid JSON: {parsed.error}
            </p>
          )}
        </div>
      }
    />
  );
};

export const CustomSchema: Story = {
  render: () => <CustomSchemaEditor />,
};
