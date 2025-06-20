import { CopyIcon } from "@phosphor-icons/react";
import {
  ChevronDownIcon,
  ChevronLeftIcon,
  ChevronRightIcon,
} from "@radix-ui/react-icons";
import { RJSFSchema } from "@rjsf/utils";
import type { Meta } from "@storybook/react";
import { useEffect, useMemo, useState } from "react";

import { Button } from "../buttons";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "../DropdownMenu";

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
