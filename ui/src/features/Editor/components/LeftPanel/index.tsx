import {
  Database,
  Disc,
  Graph,
  Icon,
  Lightning,
  RectangleDashed,
  TreeView,
} from "@phosphor-icons/react";
import { Link, useParams } from "@tanstack/react-router";
import { useReactFlow } from "@xyflow/react";
import { memo, useEffect, useState } from "react";

import { FlowLogo, Tree, TreeDataItem, IconButton } from "@flow/components";
import { UserMenu } from "@flow/features/common";
import { useShortcuts } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import type { Node } from "@flow/types";
import { getNodeIcon } from "@flow/utils/getNodeIcon";

import { ActionsList } from "./components";

type Tab = "navigator" | "actions-list" | "resources";

type Props = {
  nodes: Node[];
  isOpen: boolean;
  onOpen: (panel?: "left" | "right" | "bottom") => void;
  onNodesAdd: (node: Node[]) => void;
  isMainWorkflow: boolean;
  hasReader?: boolean;
  onNodeDoubleClick: (e: React.MouseEvent<Element>, node: Node) => void;
  selected?: Node;
};

const LeftPanel: React.FC<Props> = ({
  nodes,
  isOpen,
  onOpen,
  onNodesAdd,
  isMainWorkflow,
  hasReader,
  onNodeDoubleClick,
  selected,
}) => {
  const t = useT();
  const { workspaceId } = useParams({ strict: false });
  const [selectedTab, setSelectedTab] = useState<Tab | undefined>();
  const { fitView, getZoom, zoomTo } = useReactFlow();
  const [previousZoom, setPreviousZoom] = useState<number | undefined>(
    undefined,
  );

  useEffect(() => {
    if (previousZoom !== undefined && selected === undefined) {
      zoomTo(previousZoom, { duration: 400 });
    }
  }, [isOpen, previousZoom, selected, zoomTo]);

  useEffect(() => {
    if (selected) {
      setPreviousZoom(getZoom());
    }
  }, [selected, getZoom]);

  const [nodeId, setNodeId] = useState<string | undefined>(undefined);

  useEffect(() => {
    if (!isOpen && selectedTab) {
      setSelectedTab(undefined);
    }
  }, [isOpen, selectedTab]);

  const createTreeDataItem = (type: string, icon: Icon) => {
    return (
      nodes
        ?.filter((n) => n.type === type)
        .map((n) => ({
          id: n.id,
          name: n.data.customName || n.data.officialName || "untitled",
          icon,
          type: n.type,
        })) ?? []
    );
  };

  const treeContent: TreeDataItem[] = [
    ...createTreeDataItem("reader", Database),
    ...createTreeDataItem("writer", Disc),
    ...(nodes?.some((n) => n.type === "transformer")
      ? [
          {
            id: "transformer",
            name: t("Transformers"),
            icon: Lightning,
            children: nodes
              ?.filter((n) => n.type === "transformer")
              .map((n) => ({
                id: n.id,
                name: n.data.customName || n.data.officialName || "untitled",
                icon: Lightning,
                type: n.type,
              })),
          },
        ]
      : []),
    ...(nodes?.some((n) => n.type === "batch")
      ? [
          {
            id: "batch",
            name: t("Batch Nodes"),
            icon: RectangleDashed,
            children: nodes
              ?.filter((n) => n.type === "batch")
              .map((n) => ({
                id: n.id,
                name:
                  n.data.params?.customName ||
                  n.data.officialName ||
                  "untitled",
                icon: RectangleDashed,
                type: n.type,
                children: nodes
                  ?.filter((d) => d.parentId === n.id)
                  .map((d) => ({
                    id: d.id,
                    name:
                      d.data.customName || d.data.officialName || "untitled",
                    icon: getNodeIcon(d.type),
                  })),
              })),
          },
        ]
      : []),
    ...(nodes?.some((n) => n.type === "subworkflow")
      ? [
          {
            id: "subworkflow",
            name: t("Subworkflow"),
            icon: Graph,
            children: nodes
              ?.filter((n) => n.type === "subworkflow")
              .map((n) => ({
                id: n.id,
                name: n.data.customName || n.data.officialName || "untitled",
                icon: Graph,
                type: n.type,
              })),
          },
        ]
      : []),
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
      component: nodes && (
        <Tree
          data={treeContent}
          className="w-full shrink-0 truncate rounded px-1"
          onSelectChange={(item) => {
            setNodeId(item?.id ?? "");
          }}
          // folderIcon={Folder}
          // itemIcon={Database}
          onDoubleClick={() => {
            if (nodeId) {
              const node = nodes.find((n) => n.id === nodeId);
              if (node) {
                fitView({
                  nodes: [{ id: node.id }],
                  duration: 500,
                  padding: 2,
                });
                onNodeDoubleClick({} as React.MouseEvent, node);
              }
            }
          }}
        />
      ),
    },
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
    {
      keyBinding: { key: "r", shiftKey: true },
      callback: () => handleTabChange("resources"),
    },
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
            <Link
              to={`/workspaces/${workspaceId}`}
              className="flex shrink-0 items-center justify-center gap-2 text-lg font-semibold md:size-8 md:text-base">
              <FlowLogo className="size-7 transition-all hover:size-[30px] hover:text-[#46ce7c]" />
              <span className="sr-only">{t("Dashboard")}</span>
            </Link>
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
              dropdownPosition="right"
            />
          </nav>
        </div>
      </aside>
    </>
  );
};

export default memo(LeftPanel);
