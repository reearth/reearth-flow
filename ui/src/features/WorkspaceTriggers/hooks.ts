import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useMemo, useRef, useState } from "react";

import { usePagination } from "@flow/hooks";
import { useTrigger } from "@flow/lib/gql";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";
import { Trigger, TriggerOrderBy } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import { lastOfUrl as getTriggerId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const ref = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();
  const t = useT();
  const [openTriggerAddDialog, setOpenTriggerAddDialog] =
    useState<boolean>(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const [triggerToBeEdited, setTriggerToBeEdited] = useState<
    Trigger | undefined
  >(undefined);
  const [triggerToBeDeleted, setTriggerToBeDeleted] = useState<
    Trigger | undefined
  >(undefined);
  const { useGetTriggers, useDeleteTrigger } = useTrigger();

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const {
    page,
    totalPages,
    isFetching,
    currentPage,
    currentSortValue,
    isDebouncingSearch,
    setCurrentPage,
    setCurrentOrderDir,
    setSearchTerm,
    handleSortChange,
  } = usePagination({
    useDataQuery: useGetTriggers,
    workspaceId: currentWorkspace?.id,
    defaultOrderBy: TriggerOrderBy.UpdatedAt,
  });

  const sortOptions = [
    {
      value: `${TriggerOrderBy.UpdatedAt}_${OrderDirection.Desc}`,
      label: t("Latest Updated"),
    },
    {
      value: `${TriggerOrderBy.UpdatedAt}_${OrderDirection.Asc}`,
      label: t("Oldest Updated"),
    },
    {
      value: `${TriggerOrderBy.Description}_${OrderDirection.Asc}`,
      label: t("A To Z"),
    },
    {
      value: `${TriggerOrderBy.Description}_${OrderDirection.Desc}`,
      label: t("Z To A"),
    },
  ];

  const triggers = page?.triggers;

  const selectedTrigger = useMemo(
    () => triggers?.find((trigger) => trigger.id === tab),
    [tab, triggers],
  );

  const handleTriggerSelect = useCallback(
    (trigger: Trigger) =>
      navigate({
        to: `/workspaces/${currentWorkspace?.id}/triggers/${trigger.id}`,
      }),
    [currentWorkspace, navigate],
  );

  const handleTriggerDelete = useCallback(
    async (trigger?: Trigger) => {
      const t =
        trigger || triggers?.find((t2) => t2.id === triggerToBeDeleted?.id);
      if (!t || !currentWorkspace) return;

      await useDeleteTrigger(t.id, currentWorkspace.id);
      setTriggerToBeDeleted(undefined);
      navigate({
        to: `/workspaces/${currentWorkspace.id}/triggers`,
      });
    },
    [
      currentWorkspace,
      triggerToBeDeleted,
      triggers,
      useDeleteTrigger,
      navigate,
    ],
  );

  return {
    ref,
    triggers,
    selectedTrigger,
    triggerToBeDeleted,
    openTriggerAddDialog,
    triggerToBeEdited,
    isDebouncingSearch,
    isFetching,
    currentPage,
    currentSortValue,
    sortOptions,
    totalPages,
    handleSortChange,
    handleTriggerSelect,
    handleTriggerDelete,
    setCurrentPage,
    setCurrentOrderDir,
    setOpenTriggerAddDialog,
    setSearchTerm,
    setTriggerToBeDeleted,
    setTriggerToBeEdited,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getTriggerId(pathname);
