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
  const { useGetTriggersInfinite, useDeleteTrigger } = useTrigger();

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const { pages, hasNextPage, isFetching, fetchNextPage } =
    useGetTriggersInfinite(currentWorkspace?.id);

  const triggers: Trigger[] | undefined = useMemo(
    () =>
      pages?.reduce((triggers, page) => {
        if (page?.triggers) {
          triggers.push(...page.triggers);
        }
        return triggers;
      }, [] as Trigger[]),
    [pages],
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

  useEffect(() => {
    if (
      ref.current &&
      ref.current?.scrollHeight <= document.documentElement.clientHeight &&
      hasNextPage &&
      !isFetching
    ) {
      fetchNextPage();
    }
  }, [isFetching, hasNextPage, ref, fetchNextPage]);

  useEffect(() => {
    const handleScroll = () => {
      if (
        window.innerHeight + document.documentElement.scrollTop + 5 >=
          document.documentElement.scrollHeight &&
        !isFetching &&
        hasNextPage
      ) {
        fetchNextPage();
      }
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, [isFetching, fetchNextPage, hasNextPage]);

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
  };
};

const getTab = (pathname: string): RouteOption =>
  pathname.includes("all") ? "all" : getTriggerId(pathname);
