import { useNavigate, useRouter, useRouterState } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useCurrentWorkspace } from "@flow/stores";
import { Trigger } from "@flow/types/trigger";
import { lastOfUrl as getTriggerId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";
// Code below is only placeholder and should be replaced when backend is ready @billcookie
const sampleTriggers = [
  {
    id: "trigger-1",
    authToken: "auth-token-1",
    createdAt: "2023-01-01T12:00:00Z",
    updatedAt: "2023-01-02T12:00:00Z",
    deployment: "deployment-1",
    projectId: "project-1",
    timeInterval: "10m",
    lastTriggered: "2023-01-03T12:00:00Z",
    eventSource: "api",
  },
  {
    id: "trigger-2",
    authToken: "auth-token-2",
    createdAt: "2023-02-01T12:00:00Z",
    updatedAt: "2023-02-02T12:00:00Z",
    deployment: "deployment-2",
    projectId: null,
    timeInterval: "30m",
    lastTriggered: "2023-02-03T12:00:00Z",
    eventSource: "cms",
  },
  {
    id: "trigger-3",
    authToken: "auth-token-3",
    createdAt: "2023-03-01T12:00:00Z",
    updatedAt: "2023-03-02T12:00:00Z",
    deployment: "deployment-3",
    projectId: "project-3",
    timeInterval: "1h",
    lastTriggered: "2023-03-03T12:00:00Z",
    eventSource: "manual",
  },
];

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

  const {
    location: { pathname },
  } = useRouterState();

  const tab = getTab(pathname);

  const triggers: Trigger[] = sampleTriggers;

  const selectedTrigger = useMemo(
    () => triggers.find((trigger) => trigger.id === tab),
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
    (trigger?: Trigger) => {
      const t =
        trigger || triggers.find((t2) => t2.id === triggerToBeDeleted?.id);
      if (!t) return;
      console.log("Trigger deleted:", t);
      setTriggerToBeDeleted(undefined);
      history.go(-1);
    },
    [triggerToBeDeleted, triggers, history],
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
