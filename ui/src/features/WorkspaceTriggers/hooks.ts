import { useNavigate, useRouter, useRouterState } from "@tanstack/react-router";
import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { useCurrentWorkspace } from "@flow/stores";
import { Deployment, Workspace } from "@flow/types";
import { EventSourceType, TimeInterval, Trigger } from "@flow/types/trigger";
import { lastOfUrl as getTriggerId } from "@flow/utils";

import { RouteOption } from "../WorkspaceLeftPanel";
// Code below is only placeholder and should be replaced when backend is ready @billcookie
// Sample Deployments
const sampleDeployments: Deployment[] = [
  {
    id: "deployment-1",
    projectId: "project-1",
    projectName: "Project Alpha",
    workspaceId: "workspace-1",
    workflowUrl: "https://example.com/workflows/1",
    description: "First deployment description",
    version: "1.0.0",
    createdAt: "2023-01-01T10:00:00Z",
    updatedAt: "2023-01-02T10:00:00Z",
  },
  {
    id: "deployment-2",
    projectId: null,
    // projectName: null,
    workspaceId: "workspace-2",
    workflowUrl: "https://example.com/workflows/2",
    description: "Second deployment description",
    version: "1.1.0",
    createdAt: "2023-02-01T10:00:00Z",
    updatedAt: "2023-02-02T10:00:00Z",
  },
  {
    id: "deployment-3",
    projectId: "project-3",
    projectName: "Project Gamma",
    workspaceId: "workspace-3",
    workflowUrl: "https://example.com/workflows/3",
    description: "Third deployment description",
    version: "2.0.0",
    createdAt: "2023-03-01T10:00:00Z",
    updatedAt: "2023-03-02T10:00:00Z",
  },
];

// Sample Workspaces
const sampleWorkspaces: Workspace[] = [
  {
    id: "workspace-1",
    name: "Workspace Alpha",
    personal: false,
    members: [],
    projects: [],
  },
  {
    id: "workspace-2",
    name: "Workspace Beta",
    personal: true,
    members: [],
    projects: [],
  },
  {
    id: "workspace-3",
    name: "Workspace Gamma",
    personal: false,
    members: [],
    projects: [],
  },
];

// Sample Triggers
const sampleTriggers: Trigger[] = [
  {
    id: "trigger-1",
    authToken: "auth-token-1",
    createdAt: "2023-01-01T12:00:00Z",
    updatedAt: "2023-01-02T12:00:00Z",
    deployment: sampleDeployments[0],
    deploymentId: sampleDeployments[0].id,
    workspaceId: sampleDeployments[0].workspaceId,
    timeInterval: TimeInterval.EVERY_DAY,
    lastTriggered: "2023-01-03T12:00:00Z",
    eventSource: EventSourceType.API_DRIVEN,
  },
  {
    id: "trigger-2",
    authToken: "auth-token-2",
    createdAt: "2023-02-01T12:00:00Z",
    updatedAt: "2023-02-02T12:00:00Z",
    deployment: sampleDeployments[1],
    deploymentId: sampleDeployments[1].id,
    workspaceId: sampleDeployments[1].workspaceId,
    timeInterval: TimeInterval.EVERY_WEEK,
    lastTriggered: null,
    eventSource: EventSourceType.TIME_DRIVEN,
  },
  {
    id: "trigger-3",
    authToken: null, // Nullable
    createdAt: "2023-03-01T12:00:00Z",
    updatedAt: "2023-03-02T12:00:00Z",
    deployment: sampleDeployments[2],
    deploymentId: sampleDeployments[2].id,
    workspaceId: sampleDeployments[2].workspaceId,
    timeInterval: null, // Nullable
    lastTriggered: "2023-03-03T12:00:00Z",
    eventSource: EventSourceType.API_DRIVEN,
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
    async (trigger?: Trigger): Promise<void> => {
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
