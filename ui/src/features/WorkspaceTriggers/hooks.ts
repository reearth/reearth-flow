import { useNavigate, useRouterState } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useTrigger } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";
import { Trigger } from "@flow/types";
import { lastOfUrl as getTriggerId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const ref = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();

  const [openTriggerAddDialog, setOpenTriggerAddDialog] = useState(false);
  const [currentWorkspace] = useCurrentWorkspace();
  const [triggerToBeEdited, setTriggerToBeEdited] = useState<
    Trigger | undefined
  >(undefined);
  const [triggerToBeDeleted, setTriggerToBeDeleted] = useState<
    Trigger | undefined
  >(undefined);
  const { useGetTriggers, useDeleteTrigger } = useTrigger();
  const TRIGGERS_FETCH_RATE_PER_PAGE = 15;
  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);
  const [currentPage, setCurrentPage] = useState<number>(1);

  const { pages, refetch } = useGetTriggers(currentWorkspace?.id, {
    pageSize: TRIGGERS_FETCH_RATE_PER_PAGE,
    page: currentPage,
  });
  useEffect(() => {
    refetch();
  }, [currentPage, refetch]);
  const totalPages = pages?.totalPages as number;
  const triggers = pages?.triggers;

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
    TRIGGERS_FETCH_RATE_PER_PAGE,
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getTriggerId(pathname);
