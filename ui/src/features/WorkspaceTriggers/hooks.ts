import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useTrigger } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";
import { Trigger } from "@flow/types";
import { OrderDirection } from "@flow/types/paginationOptions";
import { lastOfUrl as getTriggerId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const ref = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();

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
  const [currentPage, setCurrentPage] = useState<number>(1);
  const [currentOrder, setCurrentOrder] = useState<OrderDirection>(
    OrderDirection.Desc,
  );
  const { page, refetch, isFetching } = useGetTriggers(currentWorkspace?.id, {
    page: currentPage,
    orderDir: currentOrder,
    orderBy: "createdAt",
  });

  useEffect(() => {
    refetch();
  }, [currentPage, currentOrder, refetch]);

  const totalPages = page?.totalPages as number;
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
    setTriggerToBeEdited,
    setOpenTriggerAddDialog,
    setTriggerToBeDeleted,
    handleTriggerSelect,
    handleTriggerDelete,
    currentPage,
    setCurrentPage,
    totalPages,
    currentOrder,
    setCurrentOrder,
    isFetching,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getTriggerId(pathname);
