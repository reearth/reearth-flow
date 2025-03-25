import {
  ChalkboardTeacher,
  Database,
  Disc,
  Graph,
  Icon,
  Lightning,
  RectangleDashed,
  TreeView,
} from "@phosphor-icons/react";
import { useNavigate, useParams } from "@tanstack/react-router";
import { memo, useCallback, useEffect, useState } from "react";

import { FlowLogo, Tree, TreeDataItem, IconButton } from "@flow/components";
import BasicBoiler from "@flow/components/BasicBoiler";
import { UserMenu } from "@flow/features/common";
import { useShortcuts } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import type { Node, NodeChange } from "@flow/types";
import { getNodeIcon } from "@flow/utils/getNodeIcon";

import { ActionsList, ProjectVariables } from "./components";

type Tab = "navigator" | "actions-list" | "resources" | "project-vars";

type Props = {
  nodes: Node[];
  isOpen: boolean;
  isMainWorkflow: boolean;
  hasReader?: boolean;
  selected?: Node;
  onOpen: (panel?: "left" | "right") => void;
  onNodesAdd: (node: Node[]) => void;
  onNodesChange: (changes: NodeChange[]) => void;
  onNodeDoubleClick: (
    e: React.MouseEvent<Element> | undefined,
    nodeId: string,
    subworkflowId?: string,
  ) => void;
};

const LeftPanel: React.FC<Props> = ({
  nodes,
  isOpen,
  isMainWorkflow,
  hasReader,
  onOpen,
  onNodesAdd,
  onNodesChange,
  onNodeDoubleClick,
}) => {
  const t = useT();
  const navigate = useNavigate();
  const { workspaceId } = useParams({ strict: false });
  const [selectedTab, setSelectedTab] = useState<Tab | undefined>();
  const [nodeId, setNodeId] = useState<string | undefined>(undefined);

  const handleNavigationToDashboard = useCallback(() => {
    navigate({ to: `/workspaces/${workspaceId}/projects` });
  }, [workspaceId, navigate]);

  useEffect(() => {
    if (!isOpen && nodeId) {
      setNodeId(undefined);
    }
  }, [isOpen, nodeId]);

  const handleTreeDataItemDoubleClick = useCallback(
    (nodeId: string) => {
      const node = nodes.find((n) => n.id === nodeId);
      if (!node) return;

      const nodeChanges: NodeChange[] = nodes.map((n) => ({
        id: n.id,
        type: "select",
        selected: n.id === nodeId,
      }));

      onNodesChange(nodeChanges);
      onNodeDoubleClick(undefined, node.id, node.data.subworkflowId);
    },
    [nodes, onNodesChange, onNodeDoubleClick],
  );

  const treeContent: TreeDataItem[] = [
    ...(createTreeDataItem("reader", Database, nodes) || []),
    ...(createTreeDataItem("writer", Disc, nodes) || []),
    ...(createTreeDataItem(
      "transformer",
      Lightning,
      nodes,
      t("Transformers"),
    ) || []),
    ...(createTreeDataItem("subworkflow", Graph, nodes, t("Subworkflows")) ||
      []),
    ...(createTreeDataItem("batch", RectangleDashed, nodes, t("Batches")) ||
      []),
  ];

  const tabs: {
    id: Tab;
    title: string;
    icon: React.ReactNode;
    component: React.ReactNode;
  }[] = [
    {
      id: "navigator",
      title: t("Canvas Navigation"),
      icon: <TreeView className="size-5" weight="thin" />,
      component:
        nodes.length !== 0 ? (
          <Tree
            data={treeContent}
            className="w-full shrink-0 select-none truncate rounded px-1"
            onSelectChange={(item) => {
              setNodeId(item?.id ?? "");
            }}
            onDoubleClick={() => {
              if (nodeId) {
                handleTreeDataItemDoubleClick(nodeId);
              }
            }}
          />
        ) : (
          <BasicBoiler
            text={t("No Nodes in Canvas")}
            className="size-4 pt-8 [&>div>p]:text-sm"
            icon={<FlowLogo className="size-12 text-accent" />}
          />
        ),
    },
    {
      id: "project-vars",
      title: t("Project Variables"),
      icon: <ChalkboardTeacher className="size-5" weight="thin" />,
      component: <ProjectVariables />,
    },
    // {
    //   id: "resources",
    //   title: t("Resources"),
    //   icon: <HardDrive className="size-5" weight="thin" />,
    //   component: <Resources />,
    // },
    {
      id: "actions-list",
      title: t("Actions list"),
      icon: <Lightning className="size-5" weight="thin" />,
      component: (
        <ActionsList
          nodes={nodes}
          onNodesAdd={onNodesAdd}
          isMainWorkflow={isMainWorkflow}
          hasReader={hasReader}
        />
      ),
    },
    // {
    //   id: "resources",
    //   title: "Resources",
    //   icon: <HardDrive className="size-5" weight="thin" />,
    //   component: <Resources />,
    // },
  ];

  const handleTabChange = (tab: Tab) => {
    if (tab === selectedTab) {
      onOpen(isOpen ? undefined : "left");
      setSelectedTab(undefined);
    } else {
      setSelectedTab(tab);
      if (!isOpen) {
        onOpen("left");
      }
    }
  };

  useShortcuts([
    {
      keyBinding: { key: "c", shiftKey: true },
      callback: () => handleTabChange("navigator"),
    },
    {
      keyBinding: { key: "a", shiftKey: true },
      callback: () => handleTabChange("actions-list"),
    },
    // {
    //   keyBinding: { key: "r", shiftKey: true },
    //   callback: () => handleTabChange("resources"),
    // },
  ]);

  return (
    <>
      <div
        className="absolute left-12 top-0 z-10 flex h-[calc(100vh-30px)] w-[300px] flex-1 flex-col gap-3 overflow-auto border-r bg-background transition-all"
        style={{
          transform: `translateX(${isOpen ? "8px" : "-100%"})`,
          transitionDuration: isOpen ? "500ms" : "300ms",
          transitionProperty: "transform",
          transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
        }}>
        <div className="flex flex-col gap-2 border-b px-4 py-2">
          <p className="text-lg dark:font-thin">
            {tabs?.find((tc) => tc.id === selectedTab)?.title}
          </p>
        </div>
        <div className="flex flex-col gap-2 overflow-auto">
          {tabs?.find((tc) => tc.id === selectedTab)?.component}
        </div>
      </div>
      <aside className="relative z-10 w-14 border-r bg-secondary">
        <div className="flex h-full flex-col">
          <nav className="flex flex-col items-center gap-5 p-3">
            <div
              className="flex shrink-0 items-center justify-center gap-2 text-lg font-semibold md:size-8 md:text-base"
              onClick={handleNavigationToDashboard}>
              <FlowLogo className="size-7 transition-all hover:size-[30px] hover:text-[#46ce7c]" />
              <span className="sr-only">{t("Dashboard")}</span>
            </div>
            {tabs.map((tab) => (
              <IconButton
                key={tab.id}
                className={`flex size-9 items-center justify-center rounded text-popover-foreground/50 transition-colors hover:text-popover-foreground md:size-8 ${selectedTab === tab.id && "bg-popover text-popover-foreground"}`}
                icon={tab.icon}
                onClick={() => handleTabChange(tab.id)}
              />
            ))}
          </nav>
          <nav className="mt-auto flex flex-col items-center gap-4 p-2">
            {/* TODO: Implement global search */}
            {/* <MagnifyingGlass
              className="size-6 cursor-pointer text-popover-foreground/50 hover:text-popover-foreground"
              weight="thin"
              onClick={() =>
                alert(
                  "Need to implement a global search and assign a shortcut as well",
                )
              }
            /> */}
            <UserMenu
              className="flex w-full justify-center"
              dropdownAlign="end"
            />
          </nav>
        </div>
      </aside>
    </>
  );
};

const createTreeDataItem = (
  type: string,
  icon: Icon,
  nodes?: Node[],
  name?: string,
) => {
  if (type === "reader" || type === "writer") {
    return (
      nodes
        ?.filter((n) => n.type === type)
        .map((n) => ({
          id: n.id,
          name:
            n.data.customizations?.customName ||
            n.data.officialName ||
            "untitled",
          icon,
          type: n.type,
        })) ?? []
    );
  }

  if (type === "transformer" || type === "subworkflow") {
    return nodes?.some((n) => n.type === type)
      ? [
          {
            id: type,
            name: name || "untitled",
            icon,
            children: nodes
              ?.filter((n) => n.type === type)
              .map((n) => ({
                id: n.id,
                name:
                  n.data.customizations?.customName ||
                  n.data.officialName ||
                  "untitled",
                icon,
                type: n.type,
              })),
          },
        ]
      : [];
  }

  if (type === "batch") {
    return nodes?.some((n) => n.type === type)
      ? [
          {
            id: type,
            name: name || "untitled",
            icon,
            children: nodes
              ?.filter((n) => n.type === type)
              .map((n) => ({
                id: n.id,
                name:
                  n.data.customizations?.customName ||
                  n.data.officialName ||
                  "untitled",
                icon,
                type: n.type,
                children: nodes
                  ?.filter((d) => d.parentId === n.id)
                  .map((d) => ({
                    id: d.id,
                    name:
                      d.data.customizations?.customName ||
                      d.data.officialName ||
                      "untitled",
                    icon: getNodeIcon(d.type),
                  })),
              })),
          },
        ]
      : [];
  }
};

export default memo(LeftPanel);
