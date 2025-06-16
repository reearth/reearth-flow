import { CaretDownIcon, PlusIcon, UserIcon } from "@phosphor-icons/react";
import { useState } from "react";

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  FlowLogo,
  Input,
} from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { useUser, useWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Role, UserMember } from "@flow/types";

import { MemberAddDialog } from "./components";

type Filter = "all" | Role;

const roles: Role[] = Object.values(Role);

const MembersSettings: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const {
    addMemberToWorkspace,
    removeMemberFromWorkspace,
    updateMemberOfWorkspace,
  } = useWorkspace();
  const { searchUser, useGetMe } = useUser();
  const [searchTerm, setSearchTerm] = useState<string>("");
  const [email, setEmail] = useState<string>("");
  const [currentFilter, setFilter] = useState<Filter>("all");
  const [error, setError] = useState<string | undefined>();

  const { me } = useGetMe();

  const filters: { id: Filter; title: string }[] = [
    { id: "all", title: t("All") },
    { id: Role.Owner, title: t("Owner") },
    { id: Role.Reader, title: t("Reader") },
    { id: Role.Maintainer, title: t("Maintainer") },
    { id: Role.Writer, title: t("Writer") },
  ];

  const members = currentWorkspace?.members?.filter(
    (m) =>
      "userId" in m &&
      (currentFilter === "all" || m.role === currentFilter) &&
      (m.user?.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        m.user?.email.toLowerCase().includes(searchTerm.toLowerCase())),
  ) as UserMember[];

  const [openMemberAddDialog, setOpenMemberAddDialog] =
    useState<boolean>(false);

  const handleAddMember = async (email: string) => {
    setError(undefined);
    if (!currentWorkspace?.id) return;

    const alreadyExists = members?.find((m) => m.user?.email === email);

    if (alreadyExists) {
      setError("User already exists");
      return;
    }

    const { user } = await searchUser(email);
    if (!user) {
      setError(t("Could not find the user"));
      return;
    }
    const { workspace } = await addMemberToWorkspace(
      currentWorkspace.id,
      user.id,
      Role.Reader,
    );

    if (!workspace) {
      return;
    }
    setEmail("");
    setOpenMemberAddDialog(false);
  };

  const handleChangeRole = async (userId: string, role: Role) => {
    if (!currentWorkspace?.id) return;
    const { workspace } = await updateMemberOfWorkspace(
      currentWorkspace.id,
      userId,
      role,
    );
    if (!workspace) {
      return;
    }
  };

  const handleRemoveMembers = async (userId: string) => {
    if (!currentWorkspace?.id) return;
    const { workspace } = await removeMemberFromWorkspace(
      currentWorkspace.id,
      userId,
    );
    if (!workspace) {
      return;
    }
  };

  return (
    <>
      <div className="flex h-[50px] items-center justify-between gap-2 border-b pb-4">
        <p className="text-lg dark:font-extralight">{t("Members Settings")}</p>
        {!currentWorkspace?.personal && (
          <Button
            className="flex gap-2"
            onClick={() => setOpenMemberAddDialog(true)}>
            <PlusIcon weight="thin" />
            <p className="text-xs dark:font-light"> {t("Add Member")}</p>
          </Button>
        )}
      </div>
      <div className="mt-4 flex max-w-[900px] flex-col gap-6">
        <div className="rounded border dark:font-extralight">
          <div className="flex h-[42px] items-center justify-between gap-2 border-b p-2">
            <div className="flex items-center gap-8">
              <Input
                className="w-[250px]"
                placeholder={t("Search...")}
                autoFocus
                onChange={(e) => setSearchTerm(e.target.value)}
              />
              <div>
                <DropdownMenu>
                  <DropdownMenuTrigger className="flex items-center gap-2">
                    <p>{filters.find((f) => f.id === currentFilter)?.title}</p>
                    <CaretDownIcon className="size-3" />
                  </DropdownMenuTrigger>
                  <DropdownMenuContent className="min-w-[70px]">
                    {filters.map((filter, idx) => (
                      <DropdownMenuItem
                        key={idx}
                        className={`h-[25px] justify-center ${filter.id === currentFilter ? "bg-accent" : undefined}`}
                        onClick={() => setFilter(filter.id)}>
                        {filter.title}
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
              </div>
            </div>
            <div className="flex items-center gap-2">
              <UserIcon weight="thin" />
              <p>{`${members?.length} ${t("Members")}`}</p>
            </div>
          </div>
          <div className="max-h-[50vh] overflow-auto">
            {members?.map((m) => (
              <div key={m.userId} className="flex gap-4 px-4 py-2">
                <p className="flex-1">{m.user?.name}</p>
                <p className="flex-1 px-4 text-sm capitalize dark:font-thin">
                  {m.role}
                </p>
                <DropdownMenu>
                  <DropdownMenuTrigger
                    disabled={m.userId === me?.id}
                    className={`flex flex-1 items-center gap-1 ${m.userId === me?.id ? "opacity-50" : ""}`}>
                    <p className="text-sm">{t("Change role")}</p>
                    <CaretDownIcon className="size-2" />
                  </DropdownMenuTrigger>
                  <DropdownMenuContent className="min-w-[70px]">
                    {roles.map((role, idx) => (
                      <DropdownMenuItem
                        key={idx}
                        onClick={() => handleChangeRole(m.userId, role)}>
                        {role}
                      </DropdownMenuItem>
                    ))}
                  </DropdownMenuContent>
                </DropdownMenu>
                <Button
                  className="h-[25px] flex-1"
                  size="sm"
                  variant="outline"
                  disabled={m.userId === me?.id}
                  onClick={() => handleRemoveMembers(m.userId)}>
                  {t("Remove")}
                </Button>
              </div>
            ))}
          </div>
          {members?.length === 0 && (
            <BasicBoiler
              className="p-8"
              text={t("No Members")}
              icon={<FlowLogo className="size-16 text-accent" />}
            />
          )}
        </div>
      </div>
      {openMemberAddDialog && (
        <MemberAddDialog
          setShowDialog={setOpenMemberAddDialog}
          email={email}
          setEmail={setEmail}
          onAddMember={handleAddMember}
          error={error}
        />
      )}
    </>
  );
};

export { MembersSettings };
