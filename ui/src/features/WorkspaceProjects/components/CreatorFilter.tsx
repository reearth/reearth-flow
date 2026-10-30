import { Combobox } from "@base-ui/react/combobox";
import { CheckIcon, MagnifyingGlassIcon, XIcon } from "@phosphor-icons/react";
import { useMemo } from "react";

import { UserAvatar } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { User } from "@flow/types";

import { useWorkspaceUsers } from "../useProjectAuthors";

type Props = {
  value: string[];
  onChange: (userIds: string[]) => void;
};

// Filters the project list by creator: any of the chosen workspace members.
const CreatorFilter: React.FC<Props> = ({ value, onChange }) => {
  const t = useT();
  const users = useWorkspaceUsers();
  const members = useMemo(
    () =>
      [...users.values()].sort((a, b) =>
        a.name.localeCompare(b.name, undefined, { sensitivity: "base" }),
      ),
    [users],
  );
  // A chosen member who has since left keeps their place until removed.
  const selected = useMemo(
    () => value.map((id) => users.get(id) ?? { id, name: t("Unknown user") }),
    [value, users, t],
  ) as User[];

  return (
    <Combobox.Root
      items={members}
      multiple
      value={selected}
      onValueChange={(users: User[]) => onChange(users.map((u) => u.id))}
      itemToStringLabel={(u: User) => u.name}
      isItemEqualToValue={(item: User, v: User) => item.id === v.id}>
      <Combobox.InputGroup className="flex h-[36px] w-[300px] cursor-text items-center gap-1 rounded-md border bg-transparent px-2 text-sm shadow-sm focus-within:ring-1 focus-within:ring-ring">
        <Combobox.Chips
          className="flex min-w-0 flex-1 items-center gap-1 overflow-x-auto"
          aria-label={selected.length ? t("Selected creators") : undefined}>
          {selected.map((user) => (
            <Combobox.Chip
              key={user.id}
              className="flex h-6 shrink-0 items-center gap-1 rounded bg-primary pr-1 pl-2 text-xs outline-hidden focus-within:ring-1 focus-within:ring-ring"
              aria-label={user.name}>
              <UserAvatar user={user} />
              <span className="max-w-[96px] truncate">{user.name}</span>
              <Combobox.ChipRemove
                className="flex size-4 items-center justify-center rounded text-muted-foreground hover:text-foreground"
                aria-label={t("Remove {{name}}", { name: user.name })}>
                <XIcon className="size-2.5" />
              </Combobox.ChipRemove>
            </Combobox.Chip>
          ))}
          <Combobox.Input
            aria-label={t("Created by")}
            placeholder={selected.length ? "" : t("Created by")}
            className="h-full min-w-12 flex-1 bg-transparent text-sm outline-hidden placeholder:text-muted-foreground dark:font-extralight"
          />
        </Combobox.Chips>
        <MagnifyingGlassIcon className="size-3.5 shrink-0 text-muted-foreground" />
      </Combobox.InputGroup>
      <Combobox.Portal>
        <Combobox.Positioner className="isolate z-50" sideOffset={4}>
          <Combobox.Popup className="max-h-[min(var(--available-height),20rem)] w-(--anchor-width) origin-(--transform-origin) overflow-y-auto rounded-lg border bg-primary/90 p-1 shadow-md backdrop-blur-md transition-[opacity,transform,scale] duration-150 data-ending-style:scale-95 data-ending-style:opacity-0 data-starting-style:scale-95 data-starting-style:opacity-0">
            <Combobox.Empty>
              <p className="px-4 py-1.5 text-sm text-muted-foreground">
                {t("No members found")}
              </p>
            </Combobox.Empty>
            <Combobox.List className="flex flex-col gap-0.5">
              {(user: User) => (
                <Combobox.Item
                  key={user.id}
                  value={user}
                  className="flex cursor-default items-center justify-between gap-2 rounded-md px-4 py-1.5 text-sm outline-hidden select-none data-highlighted:bg-accent data-selected:bg-accent/60 dark:font-light">
                  <span className="flex min-w-0 items-center gap-2">
                    <UserAvatar user={user} />
                    <span className="truncate">{user.name}</span>
                  </span>
                  <Combobox.ItemIndicator>
                    <CheckIcon className="size-3.5" />
                  </Combobox.ItemIndicator>
                </Combobox.Item>
              )}
            </Combobox.List>
          </Combobox.Popup>
        </Combobox.Positioner>
      </Combobox.Portal>
    </Combobox.Root>
  );
};

export { CreatorFilter };
