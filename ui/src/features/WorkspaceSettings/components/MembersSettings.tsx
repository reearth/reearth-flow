import { CaretDown, User } from "@phosphor-icons/react";
import { useState } from "react";

import {
  Button,
  Checkbox,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { useT } from "@flow/providers";
import { useCurrentWorkspace } from "@flow/stores";
import { Role } from "@flow/types";

type Filter = "all" | Role;

const roles = ["admin", "reader", "writer"];

const MembersSettings: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();

  const [currentFilter, setFilter] = useState<Filter>("all");

  const filters: { id: Filter; title: string }[] = [
    { id: "all", title: t("All") },
    { id: "admin", title: t("Admin") },
    { id: "reader", title: t("Reader") },
    { id: "writer", title: t("Writer") },
  ];

  const members =
    currentWorkspace?.members?.filter(m =>
      currentFilter !== "all" ? m.role === currentFilter : true,
    ) ?? [];

  const [selectedMembers, setSelectedMembers] = useState<string[]>([]);

  return (
    <div>
      <div className="flex flex-col gap-6 mt-4 max-w-[700px]">
        <div className="flex justify-between">
          <p className="text-lg font-extralight">{t("Members Settings")}</p>
          <Button>{t("Add Members")}</Button>
        </div>
        <div className="border border-zinc-700 rounded font-extralight">
          <div className="flex justify-between items-center gap-2 p-2 border-b border-zinc-700 h-[42px]">
            <div className="flex items-center gap-2">
              <Checkbox
                className="border-zinc-600 mx-2"
                checked={selectedMembers.length === members.length}
                onClick={() =>
                  setSelectedMembers(
                    selectedMembers.length !== members.length ? members.map(m => m.userId) : [],
                  )
                }
              />
              <User />
              <p>
                {selectedMembers.length
                  ? `${selectedMembers.length} ${selectedMembers.length === 1 ? t("member selected") : t("members selected")}`
                  : `${members.length} ${t("Members")}`}
              </p>
            </div>
            {selectedMembers.length > 0 && (
              <div className="flex gap-4">
                <DropdownMenu>
                  <DropdownMenuTrigger className="flex items-center gap-1">
                    <p className="text-sm">{t("Change role")}</p>
                    <CaretDown className="w-2 h-2" />
                  </DropdownMenuTrigger>

                  <DropdownMenuContent className="min-w-[70px]">
                    {roles.map((role, idx) => (
                      <DropdownMenuItem key={idx} onClick={() => console.log(role)}>
                        {role}
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
                <Button className="h-[25px]" size="sm" variant="destructive">
                  {t("Remove selected")}
                </Button>
              </div>
            )}
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
            {members.map(member => (
              <div key={member.userId} className="flex items-center gap-4 px-4 py-2">
                <Checkbox
                  className="border-zinc-600"
                  checked={selectedMembers.includes(member.userId)}
                  onClick={() =>
                    setSelectedMembers(prev =>
                      prev.includes(member.userId)
                        ? [...prev.filter(pm => pm !== member.userId)]
                        : [...prev, member.userId],
                    )
                  }
                />
                <p>{member.user.name}</p>
                <p className="px-4 font-thin capitalize text-sm">{member.role}</p>
              </div>
            ))}
          </div>
        </div>
        <Button className="self-end">{t("Save")}</Button>
      </div>
    </div>
  );
};

export { MembersSettings };
