import { useNavigate, useRouter, useRouterState } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useTrigger } from "@flow/lib/gql";
import { useCurrentWorkspace } from "@flow/stores";
import { Trigger } from "@flow/types";
import { lastOfUrl as getTriggerId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";

export default () => {
  const ref = useRef<HTMLDivElement>(null);
  const navigate = useNavigate();
  const { history } = useRouter();

  const [openTriggerAddDialog, setOpenTriggerAddDialog] = useState(false);
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

  const { triggers } = useGetTriggers(currentWorkspace?.id);

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
      history.go(-1);
    },
    [currentWorkspace, triggerToBeDeleted, triggers, history, useDeleteTrigger],
  );

  useEffect(() => {
    if (
      ref.current &&
      ref.current?.scrollHeight <= document.documentElement.clientHeight
    ) {
      console.log("Fetch next page placeholder");
    }
  }, [ref]);

  useEffect(() => {
    const handleScroll = () => {
      if (
        window.innerHeight + document.documentElement.scrollTop + 5 >=
        document.documentElement.scrollHeight
      ) {
        console.log("Fetch next page placeholder");
      }
    };
    window.addEventListener("scroll", handleScroll);
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

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
