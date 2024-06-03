import { useState } from "react";
import { useReactFlow } from "reactflow";

import {
  Command,
  CommandList,
  CommandInput,
  CommandItem,
  CommandGroup,
  CommandSeparator,
} from "@flow/components/Command";
import { useT } from "@flow/lib/i18n";

const commandClasses =
  "[&_[cmdk-group-heading]]:px-2 [&_[cmdk-group-heading]]:font-medium [&_[cmdk-group-heading]]:text-muted-foreground [&_[cmdk-group]:not([hidden])_~[cmdk-group]]:pt-0 [&_[cmdk-group]]:px-2 [&_[cmdk-input-wrapper]_svg]:h-5 [&_[cmdk-input-wrapper]_svg]:w-5 [&_[cmdk-input]]:h-12 [&_[cmdk-item]]:px-2 [&_[cmdk-item]]:py-3 [&_[cmdk-item]_svg]:h-5 [&_[cmdk-item]_svg]:w-5";

const valueInSearch = (searchTerm: string, values?: string[]) =>
  !!values?.find(
    value => searchTerm.length && value?.toLowerCase().includes(searchTerm.toLowerCase()),
  );

const CanvasSearch: React.FC = () => {
  const t = useT();
  const reactFlowInstance = useReactFlow();
  const nodes = reactFlowInstance.getNodes();

  const [searchValue, setSearchValue] = useState("");

  const filteredReaders = nodes.filter(
    node => node.type === "reader" && valueInSearch(searchValue, [node.data.name]),
  );
  const filteredWriters = nodes.filter(
    node => node.type === "writer" && valueInSearch(searchValue, [node.data.name]),
  );
  const filteredTransformers = nodes.filter(
    node => node.type === "transformer" && valueInSearch(searchValue, [node.data.name]),
  );

  //   const edges = reactFlowInstance.getEdges();

  return (
    <Command className={commandClasses} shouldFilter={false}>
      <CommandInput
        placeholder={t("search workflow...")}
        autoFocus
        onValueChange={setSearchValue}
      />
      <CommandList className="border-none">
        {searchValue.length ? (
          <>
            <div className="border-t border-zinc-700" />
            {/* <CommandEmpty>{t("No results found.")}</CommandEmpty> */}
            <CommandGroup heading="Readers">
              {filteredReaders.map(n => {
                return (
                  <CommandItem key={n.id} value={n.data.name}>
                    {n.data.name}
                  </CommandItem>
                );
              })}
            </CommandGroup>
            <CommandSeparator />
            <CommandGroup heading="Writers">
              {filteredWriters.map(n => {
                return (
                  <CommandItem key={n.id} value={n.data.name}>
                    {n.data.name}
                  </CommandItem>
                );
              })}
            </CommandGroup>
            <CommandSeparator />
            <CommandGroup heading="Transformers">
              {filteredTransformers.map(n => {
                return (
                  <CommandItem key={n.id} value={n.data.name}>
                    {n.data.name}
                  </CommandItem>
                );
              })}
            </CommandGroup>
          </>
        ) : null}
      </CommandList>
    </Command>
  );
};

export { CanvasSearch };
