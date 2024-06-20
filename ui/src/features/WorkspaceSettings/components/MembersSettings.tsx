import { CaretDown, User } from "@phosphor-icons/react";
import { useState } from "react";

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  Input,
} from "@flow/components";
import { useUser, useWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Role, UserMember } from "@flow/types";

type Filter = "all" | Role;

const roles: Role[] = Object.values(Role);

const MembersSettings: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const { addMemberToWorkspace, removeMemberFromWorkspace, updateMemberOfWorkspace } =
    useWorkspace();
  const { searchUser, getMe } = useUser();
  const [email, setEmail] = useState<string>("");
  const [currentFilter, setFilter] = useState<Filter>("all");
  const [error, setError] = useState<string | undefined>();

  const me = getMe();

  const filters: { id: Filter; title: string }[] = [
    { id: "all", title: t("All") },
    { id: Role.Owner, title: t("Owner") },
    { id: Role.Reader, title: t("Reader") },
    { id: Role.Maintainer, title: t("Maintainer") },
    { id: Role.Writer, title: t("Writer") },
  ];

  const members = currentWorkspace?.members?.filter(
    m => "userId" in m && (currentFilter === "all" || m.role === currentFilter),
  ) as UserMember[];

  const handleAddMember = async (email: string) => {
    setError(undefined);
    if (!currentWorkspace?.id) return;
    const { user } = await searchUser(email);
    if (!user) {
      setError(t("Could not find the user"));
      return;
    }
    const { workspace } = await addMemberToWorkspace(currentWorkspace.id, user.id, Role.Reader);

    if (!workspace) {
      setError(t("Failed to add member"));
      return;
    }
    setEmail("");
  };

  const handleChangeRole = async (userId: string, role: Role) => {
    setError(undefined);
    if (!currentWorkspace?.id) return;
    const { workspace } = await updateMemberOfWorkspace(currentWorkspace.id, userId, role);
    if (!workspace) {
      setError(t("Failed to change role of the member"));
      return;
    }
  };

  const handleRemoveMembers = async (userId: string) => {
    setError(undefined);
    if (!currentWorkspace?.id) return;
    const { workspace } = await removeMemberFromWorkspace(currentWorkspace.id, userId);
    if (!workspace) {
      setError(t("Failed to remove member"));
      return;
    }
  };

  return (
    <div>
      <div className="flex flex-col gap-6 mt-4 max-w-[800px]">
        <div className="flex justify-between">
          <p className="text-lg font-extralight">{t("Members Settings")}</p>
        </div>
        <div className="flex justify-between items-center">
          {/* TODO: This will be a dialog component */}
          <Input
            className="w-2/4"
            placeholder={t("Enter email")}
            value={email}
            disabled={currentWorkspace?.personal}
            onChange={e => setEmail(e.target.value)}
          />
          <Button
            onClick={() => handleAddMember(email)}
            disabled={!email || currentWorkspace?.personal}>
            {t("Add Member")}
          </Button>
        </div>
        <div className="border border-zinc-700 rounded font-extralight">
          <div className="flex justify-between items-center gap-2 p-2 border-b border-zinc-700 h-[42px]">
            <div className="flex items-center gap-2">
              <User weight="thin" />
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
            {members.map(m => (
              <div key={m.userId} className="flex gap-4 px-4 py-2">
                <p className="flex-1">{m.user?.name}</p>
                <p className="flex-1 px-4 font-thin capitalize text-sm">{m.role}</p>
                <DropdownMenu>
                  <DropdownMenuTrigger
                    disabled={m.userId === me.me?.id}
                    className={`flex-1 flex items-center gap-1 ${m.userId === me.me?.id ? "opacity-50" : ""}`}>
                    <p className="text-sm">{t("Change role")}</p>
                    <CaretDown className="w-2 h-2" />
                  </DropdownMenuTrigger>

                  <DropdownMenuContent className="min-w-[70px]">
                    {roles.map((role, idx) => (
                      <DropdownMenuItem key={idx} onClick={() => handleChangeRole(m.userId, role)}>
                        {role}
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
                <Button
                  className="flex-1 h-[25px]"
                  size="sm"
                  variant="outline"
                  disabled={m.userId === me.me?.id}
                  onClick={() => handleRemoveMembers(m.userId)}>
                  {t("Remove")}
                </Button>
              </div>
            ))}
          </div>
        </div>
        <p className="text-sm text-red-400">{error}</p>
      </div>
    </div>
  );
};

export { MembersSettings };
