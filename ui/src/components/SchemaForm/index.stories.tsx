import { CopyIcon } from "@phosphor-icons/react";
import {
  ChevronDownIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
} from "@radix-ui/react-icons";
import { RJSFSchema } from "@rjsf/utils";
import validator from "@rjsf/validator-ajv8";
import type { Meta } from "@storybook/react";
import { useEffect, useMemo, useState } from "react";

import { Button } from "../buttons";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "../DropdownMenu";

import { ThemedForm } from "./ThemedForm";

import { SchemaForm } from ".";

const meta = {
  component: SchemaForm,
  parameters: {
    layout: "centered",
  },
  tags: ["autodocs"],
  argTypes: {},
} satisfies Meta<typeof SchemaForm>;

export default meta;

const commonArgs: {
  schema: RJSFSchema;
} = {
  schema: {
    $schema: "http://json-schema.org/draft-07/schema#",
    definitions: {
      Attribute: {
        type: "string",
      },
    },
    properties: {
      groupBy: {
        items: {
          $ref: "#/definitions/Attribute",
        },
        type: ["array", "null"],
      },
      outputAttribute: {
        $ref: "#/definitions/Attribute",
      },
    },
    required: ["outputAttribute"],
    title: "AreaOnAreaOverlayerParam",
    type: "object",
  },
};

const fetcher = async (url: string) => {
  const response = await fetch(url);

  if (!response.ok) {
    throw new Error("response not ok");
  }
  return await response.json();
};

export const RangeWidgetDemo = () => {
  const rangeSchema: RJSFSchema = {
    $schema: "http://json-schema.org/draft-07/schema#",
    title: "Range Widget Demo",
    type: "object",
    properties: {
      basicRange: {
        type: "number",
        title: "Basic Range (0-100)",
        description: "A simple range slider",
        minimum: 0,
        maximum: 100,
        default: 50,
      },
      precisionRange: {
        type: "number",
        title: "Precision Range (0-10, step 0.1)",
        description: "Range with decimal precision",
        minimum: 0,
        maximum: 10,
        multipleOf: 0.1,
        default: 5.5,
      },
      temperatureRange: {
        type: "integer",
        title: "Temperature (-20°C to 40°C)",
        description: "Temperature range with negative values",
        minimum: -20,
        maximum: 40,
        default: 20,
      },
      percentageRange: {
        type: "number",
        title: "Percentage (0-1, step 0.01)",
        description: "Percentage as decimal (0.0 to 1.0)",
        minimum: 0,
        maximum: 1,
        multipleOf: 0.01,
        default: 0.75,
      },
      normalNumber: {
        type: "number",
        title: "Normal Number Input (for comparison)",
        description: "This should render as a number input",
        minimum: 0,
        maximum: 100,
        default: 25,
      },
    },
  };

  const rangeUiSchema = {
    basicRange: {
      "ui:widget": "range",
    },
    precisionRange: {
      "ui:widget": "range",
    },
    temperatureRange: {
      "ui:widget": "range",
    },
    percentageRange: {
      "ui:widget": "range",
    },
    // normalNumber doesn't specify a widget, so it uses the default NumberInput
  };

  return (
    <div className="w-[600px] rounded border p-4">
      <h3 className="mb-4 text-lg font-semibold">
        Range Widget Component Demo
      </h3>
      <ThemedForm
        schema={rangeSchema}
        uiSchema={rangeUiSchema}
        validator={validator}
        onChange={(data) => console.log("Range data:", data)}
      />
    </div>
  );
};

export const NumberInputDemo = () => {
  const numberSchema: RJSFSchema = {
    $schema: "http://json-schema.org/draft-07/schema#",
    title: "Number Input Demo",
    type: "object",
    properties: {
      simpleNumber: {
        type: "number",
        title: "Simple Number",
        description: "A basic number input",
      },
      integerOnly: {
        type: "integer",
        title: "Integer Only",
        description: "Integer input with min/max constraints",
        minimum: 0,
        maximum: 100,
      },
      stepNumber: {
        type: "number",
        title: "Number with Step",
        description: "Number input with custom step",
        minimum: 0,
        maximum: 10,
        multipleOf: 0.5,
        default: 2.5,
      },
      textInput: {
        type: "string",
        format: "text",
        title: "Text Input (for comparison)",
        description: "This should render as a text input",
      },
      colorInput: {
        type: "string",
        format: "color",
        title: "Color Input (for comparison)",
        description: "This should render as a color picker",
        default: "#ff0000",
      },
    },
    required: ["simpleNumber"],
  };

  return (
    <div className="w-[600px] rounded border p-4">
      <h3 className="mb-4 text-lg font-semibold">
        Number Input Component Demo
      </h3>
      <SchemaForm
        schema={numberSchema}
        onChange={(data) => console.log("Form data:", data)}
      />
    </div>
  );
};

export const Default = () => {
  const [actions, setActions] = useState<any[]>([]);
  const [selectedAction, setSelectedAction] = useState();
  const [schema, setSchema] = useState<RJSFSchema>(commonArgs.schema);
  const [showSchema, setShowSchema] = useState<boolean>(false);

  useEffect(() => {
    (async () => {
      const data = await fetcher("http://localhost:8080/actions");
      setActions(data);
    })();
  }, []);

  useEffect(() => {
    (async () => {
      const { parameter } = await fetcher(
        `http://localhost:8080/actions/${selectedAction}`,
      );
      setSchema(parameter);
    })();
  }, [selectedAction, setSchema]);

  const selectedIndex = useMemo(
    () => actions.findIndex((el: any) => el.name === selectedAction),
    [actions, selectedAction],
  );

  return (
    <div className="flex w-[80vw] flex-col gap-2">
      <div className="flex items-center justify-between rounded border p-2">
        <div>Action Name: {selectedAction ? selectedAction : "Default"}</div>
        <div className="flex w-1/2 items-center justify-between gap-2">
          <Button
            size="sm"
            variant="outline"
            disabled={selectedIndex <= 0}
            onClick={() => setSelectedAction(actions[selectedIndex - 1].name)}>
            <ChevronLeftIcon />
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={selectedIndex >= actions.length}
            onClick={() => setSelectedAction(actions[selectedIndex + 1].name)}>
            <ChevronRightIcon />
          </Button>
          <DropdownMenu modal={true}>
            <DropdownMenuTrigger className="flex h-8 items-center rounded border bg-background px-1 hover:bg-accent">
              <p className="text-sm"> Select Action</p>
              <ChevronDownIcon />
            </DropdownMenuTrigger>
            <DropdownMenuContent className="h-96 overflow-auto" align="center">
              {actions.map(({ name }) => (
                <DropdownMenuItem
                  onClick={() => setSelectedAction(name)}
                  key={name}>
                  {name}
                </DropdownMenuItem>
              ))}
            </DropdownMenuContent>
          </DropdownMenu>
          <Button variant="outline" disabled size="sm" className="text-sm">
            Enter Custom Schema
          </Button>
          <Button
            variant="outline"
            size="sm"
            className="text-sm"
            onClick={() => setShowSchema(!showSchema)}>
            {showSchema ? "Hide" : "Show"} Schema
          </Button>
        </div>
      </div>
      {showSchema && (
        <pre className="relative rounded border bg-card p-2 text-xs">
          {JSON.stringify(schema, null, 2)}
          <Button
            size="sm"
            variant="outline"
            className="absolute top-0 right-0 mt-2 mr-2"
            onClick={() => {
              navigator.clipboard.writeText(JSON.stringify(schema, null, 2));
            }}>
            <CopyIcon />
          </Button>
        </pre>
      )}
      <div className="rounded border p-2">
        <SchemaForm schema={schema} onChange={() => console.log("change!")} />
      </div>
    </div>
  );
};
