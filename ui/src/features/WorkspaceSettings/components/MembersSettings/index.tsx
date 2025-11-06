import { CaretDownIcon, PlusIcon } from "@phosphor-icons/react";
import { ColumnDef } from "@tanstack/react-table";
import { useCallback, useState } from "react";

import {
  Button,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
  DataTable as Table,
} from "@flow/components";
import { useUser, useWorkspace } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Role, UserMember } from "@flow/types";

import { MemberAddDialog } from "./components";

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
  const [currentFilter, setFilter] = useState<string>("all");
  const [error, setError] = useState<string | undefined>();

  const handleSortChange = useCallback((newSortValue: string) => {
    setFilter(newSortValue);
  }, []);

  const { me } = useGetMe();

  const filters: { value: string; label: string }[] = [
    { value: "all", label: t("All") },
    { value: Role.Owner, label: t("Owner") },
    { value: Role.Reader, label: t("Reader") },
    { value: Role.Maintainer, label: t("Maintainer") },
    { value: Role.Writer, label: t("Writer") },
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

  const columns: ColumnDef<UserMember>[] = [
    {
      accessorKey: "user.name",
      header: t("Name"),
    },
    {
      accessorKey: "role",
      header: t("Role"),
    },
    {
      accessorKey: "actions",
      header: t("Actions"),
      cell: (row) => (
        <div className="flex items-center gap-8">
          <div key={row.row.original.userId} className="flex gap-4 py-2">
            <DropdownMenu>
              <DropdownMenuTrigger
                disabled={row.row.original.userId === me?.id}
                className={`flex flex-1 items-center gap-1 ${row.row.original.userId === me?.id ? "opacity-50" : ""}`}>
                <p className="text-sm">{t("Change role")}</p>
                <CaretDownIcon className="size-2" />
              </DropdownMenuTrigger>
              <DropdownMenuContent className="min-w-[70px]">
                {roles.map((role, idx) => (
                  <DropdownMenuItem
                    key={idx}
                    onClick={() =>
                      handleChangeRole(row.row.original.userId, role)
                    }>
                    {role}
                  </DropdownMenuItem>
                ))}
              </DropdownMenuContent>
            </DropdownMenu>
          </div>
          <Button
            className="h-[25px]"
            size="sm"
            variant="outline"
            disabled={row.row.original.userId === me?.id}
            onClick={() => handleRemoveMembers(row.row.original.userId)}>
            {t("Remove")}
          </Button>
        </div>
      ),
    },
  ];

  return (
    <>
      <div className="flex flex-1 flex-col gap-1">
        <div className="flex h-[50px] items-center justify-between gap-2 border-b pb-4">
          <p className="text-lg dark:font-extralight">
            {t("Members Settings")}
          </p>
          {!currentWorkspace?.personal && (
            <Button
              className="flex gap-2"
              onClick={() => setOpenMemberAddDialog(true)}>
              <PlusIcon weight="thin" />
              <p className="text-xs dark:font-light"> {t("Add Member")}</p>
            </Button>
          )}
        </div>

        <div className="h-full flex-1 overflow-hidden">
          <Table
            columns={columns}
            data={members}
            selectColumns
            showFiltering
            showOrdering
            currentSortValue={currentFilter}
            sortOptions={filters}
            onSortChange={handleSortChange}
            setSearchTerm={setSearchTerm}
          />
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
