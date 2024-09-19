import { CaretDown, Plugs } from "@phosphor-icons/react";
import { useState } from "react";

import {
  Button,
  Checkbox,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Role } from "@flow/types";
import { IntegrationMember } from "@flow/types/integration";

type Filter = "all" | Role;

const roles: Role[] = Object.values(Role);

const IntegrationsSettings: React.FC = () => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();

  const [currentFilter, setFilter] = useState<Filter>("all");

  const filters: { id: Filter; title: string }[] = [
    { id: "all", title: t("All") },
    { id: Role.Owner, title: t("Owner") },
    { id: Role.Reader, title: t("Reader") },
    { id: Role.Reader, title: t("Maintainer") },
    { id: Role.Writer, title: t("Writer") },
  ];

  const integrations: IntegrationMember[] =
    (currentWorkspace?.members?.filter(
      (m) =>
        "integration" in m &&
        (currentFilter !== "all" ? m.integrationRole === currentFilter : true),
    ) as IntegrationMember[]) ?? [];

  const [selectedIntegrations, setSelectedIntegrations] = useState<string[]>(
    [],
  );

  return (
    <div>
      <div className="mt-4 flex max-w-[800px] flex-col gap-6">
        <div className="flex justify-between">
          <p className="text-lg dark:font-extralight">
            {t("Integrations Settings")}
          </p>
          <Button>{t("Connect Integration")}</Button>
        </div>
        <div className="rounded border dark:font-extralight">
          <div className="flex h-[42px] items-center justify-between gap-2 border-b p-2">
            <div className="flex items-center gap-2">
              <Checkbox
                className="mx-2"
                checked={selectedIntegrations.length === integrations.length}
                onClick={() =>
                  setSelectedIntegrations(
                    selectedIntegrations.length !== integrations.length
                      ? integrations.map((m) => m.id)
                      : [],
                  )
                }
              />
              <Plugs weight="thin" />
              <p>
                {selectedIntegrations.length
                  ? `${selectedIntegrations.length} ${selectedIntegrations.length === 1 ? t("member selected") : t("members selected")}`
                  : `${integrations.length} ${t("Integrations")}`}
              </p>
            </div>
            {selectedIntegrations.length > 0 && (
              <div className="flex gap-4">
                <DropdownMenu>
                  <DropdownMenuTrigger className="flex items-center gap-1">
                    <p className="text-sm">{t("Change role")}</p>
                    <CaretDown className="size-2" />
                  </DropdownMenuTrigger>

                  <DropdownMenuContent className="min-w-[70px]">
                    {roles.map((role, idx) => (
                      <DropdownMenuItem
                        key={idx}
                        onClick={() => console.log(role)}>
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
                  <p>{filters.find((f) => f.id === currentFilter)?.title}</p>
                  <CaretDown className="size-3" />
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
          <div className="max-h-[50vh] overflow-auto">
            {integrations.map((integration) => (
              <div
                key={integration.id}
                className="flex items-center gap-4 px-4 py-2">
                <Checkbox
                  checked={selectedIntegrations.includes(integration.id)}
                  onClick={() =>
                    setSelectedIntegrations((prev) =>
                      prev.includes(integration.id)
                        ? [...prev.filter((pm) => pm !== integration.id)]
                        : [...prev, integration.id],
                    )
                  }
                />
                <p>{integration.integration?.name}</p>
                <p className="px-4 text-sm dark:font-thin capitalize">
                  {integration.integrationRole}
                </p>
              </div>
            ))}
          </div>
        </div>
        <Button className="self-end">{t("Save")}</Button>
      </div>
    </div>
  );
};

export { IntegrationsSettings };
