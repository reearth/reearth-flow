import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useMemo, useState } from "react";

import { useTrigger } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";
import { Trigger } from "@flow/types";
import { lastOfUrl as getTriggerId } from "@flow/utils";

import usePagination from "../hooks/usePagination";
import { RouteOption } from "../WorkspaceLeftPanel";

const TRIGGERS_FETCH_RATE = 15;

export default () => {
  const navigate = useNavigate();
  const [openTriggerAddDialog, setOpenTriggerAddDialog] = useState(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const [triggerToBeEdited, setTriggerToBeEdited] = useState<
    Trigger | undefined
  >(undefined);
  const [triggerToBeDeleted, setTriggerToBeDeleted] = useState<
    Trigger | undefined
  >(undefined);
  const [currentPage, setCurrentPage] = useState<number>(0);
  const { useGetTriggersInfinite, useDeleteTrigger } = useTrigger();

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const { pages, hasNextPage, isFetching, fetchNextPage, isFetchingNextPage } =
    useGetTriggersInfinite(currentWorkspace?.id, TRIGGERS_FETCH_RATE);

  const triggers: Trigger[] | undefined = useMemo(
    () => pages?.[currentPage]?.triggers,
    [pages, currentPage],
  );

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
    },
    [currentWorkspace, triggerToBeDeleted, triggers, useDeleteTrigger],
  );

  const { totalPages, handleNextPage, handlePrevPage, canGoNext } =
    usePagination<Trigger>(
      TRIGGERS_FETCH_RATE,
      hasNextPage,
      isFetchingNextPage,
      pages,
      fetchNextPage,
      currentPage,
      setCurrentPage,
    );
  return {
    triggers,
    totalPages,
    currentPage,
    hasNextPage: canGoNext,
    isFetching,
    isFetchingNextPage,
    handleNextPage,
    handlePrevPage,
    selectedTrigger,
    triggerToBeDeleted,
    openTriggerAddDialog,
    triggerToBeEdited,
    setTriggerToBeEdited,
    setOpenTriggerAddDialog,
    setTriggerToBeDeleted,
    handleTriggerSelect,
    handleTriggerDelete,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getTriggerId(pathname);
