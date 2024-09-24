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
  const status = response.status;
  if (status != 200) {
    throw new Error(`status not 200. received ${status}`);
  }
  return await response.json();
};

export const Default = () => {
  const [actions, setActions] = useState<any[]>([]);
  const [selectedAction, setSelectedAction] = useState();
  const [schema, setSchema] = useState<RJSFSchema>(commonArgs.schema);

  useEffect(() => {
    (async () => {
      const data = await fetcher("http://localhost:8080/actions");
      setActions(data);
    })();
  }, []);

  useEffect(() => {
    (async () => {
      const data = await fetcher(
        `http://localhost:8080/actions/${selectedAction}`,
      );
      setSchema(data.parameter);
    })();
  }, [selectedAction, setSchema]);

  const selectedIndex = useMemo(
    () => actions.findIndex((el: any) => el.name === selectedAction),
    [actions, selectedAction],
  );

  console.log(selectedIndex);

  const selectNext = () => console.log();

  return (
    <div className="flex w-[80vw] flex-col gap-2">
      <div className="flex items-center justify-between rounded border p-2">
        <div>Action Name: {selectedAction ? selectedAction : "Default"}</div>
        <div className="flex w-1/3 items-center justify-between gap-2">
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
        </div>
      </div>
      <div className="rounded border p-2">
        <SchemaForm schema={schema} />
      </div>
    </div>
  );
};
