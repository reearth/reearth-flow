import { CaretDown, User } from "@phosphor-icons/react";
import { useState } from "react";

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Input,
  Label,
} from "@flow/components";
import { useT } from "@flow/providers";
import { useCurrentWorkspace } from "@flow/stores";
import { Role } from "@flow/types";

type Filter = "all" | Role;

const MembersSettings: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const members = currentWorkspace?.members ?? [];

  const filters: { id: Filter; title: string }[] = [
    { id: "all", title: t("All") },
    { id: "admin", title: t("Admin") },
    { id: "reader", title: t("Reader") },
    { id: "writer", title: t("Writer") },
  ];

  const [currentFilter, setFilter] = useState<Filter>("all");

  return (
    <div>
      <div className="flex flex-col gap-6 mt-4 max-w-[700px]">
        <div className="flex justify-between">
          <p className="text-lg font-extralight">{t("Members Settings")}</p>
          <Button>{t("Add Members")}</Button>
        </div>
        <div className="border border-zinc-700 rounded font-extralight">
          <div className="flex justify-between items-center gap-2 p-2 border-b border-zinc-700">
            <div className="flex items-center gap-2">
              <User />
              <p>{`${members.length} ${t("Members")}`}</p>
            </div>
            <div>
              <DropdownMenu>
                <DropdownMenuTrigger className="flex items-center gap-2">
                  <p>{filters.find(f => f.id === currentFilter)?.title}</p>
                  <CaretDown className="w-3 h-3" />
                </DropdownMenuTrigger>

                <DropdownMenuContent className="min-w-[70px]">
                  {filters.map((filter, idx) => (
                    <DropdownMenuItem
                      key={idx}
                      className={`justify-center h-[25px] ${filter.id === currentFilter ? "bg-zinc-700/50" : undefined}`}
                      onClick={() => setFilter(filter.id)}>
                      {filter.title}
                    </DropdownMenuItem>
                  ))}
                </DropdownMenuContent>
              </DropdownMenu>
            </div>
          </div>
          <div className="max-h-[50vh] overflow-auto">
            {members
              .filter(m => (currentFilter !== "all" ? m.role === currentFilter : true))
              ?.map(member => (
                <div key={member.userId} className="flex items-center gap-8 px-4 py-2">
                  <p>{member.user.name}</p>
                  <p className="font-thin capitalize text-sm">{member.role}</p>
                </div>
              ))}
          </div>
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="workspace-name">{t("Workspace Name")}</Label>
          <Input
            id="workspace-name"
            placeholder={t("Workspace Name")}
            // defaultValue={currentWorkspace?.name}
          />
        </div>
        <div className="flex flex-col gap-2">
          <Label htmlFor="workspace-description">{t("Workspace Description")}</Label>
          <Input
            id="workspace-description"
            placeholder={t("Workspace Description")}
            // defaultValue={currentWorkspace?.description}
          />
        </div>
        <Button className="self-end">{t("Save")}</Button>
      </div>
    </div>
  );
};

export { MembersSettings };
