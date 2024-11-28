import {
  Circle,
  Database,
  Disc,
  HardDrive,
  Lightning,
  MagnifyingGlass,
  Note,
  Plus,
  Lock,
  RectangleDashed,
  TreeView,
} from "@phosphor-icons/react";
import { Link, useParams } from "@tanstack/react-router";
import { memo, useEffect, useState } from "react";

import { FlowLogo, Tree, TreeDataItem, IconButton } from "@flow/components";
import { UserNavigation } from "@flow/features/WorkspaceTopNavigation/components";
import { useShortcuts } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import type { Node } from "@flow/types";

import { ActionsList, Resources } from "./components";

import { useLocker } from "../../useInteractionLocker";

type Tab = "navigator" | "actions-list" | "resources";

type Props = {
  nodes: Node[];
  isOpen: boolean;
  onOpen: (panel?: "left" | "right" | "bottom") => void;
  onNodesChange: (nodes: Node[]) => void;
  onNodeLocking: (nodeId: string, options?: { source: string }) => void;
};

const LeftPanel: React.FC<Props> = ({
  nodes,
  isOpen,
  onOpen,
  onNodesChange,
  onNodeLocking,
}) => {
  const t = useT();
  const { workspaceId } = useParams({ strict: false });
  const [selectedTab, setSelectedTab] = useState<Tab | undefined>();

  const [_content, setContent] = useState("Admin Page");

  const {
    interactionLockedNodes,
    lockNodeInteraction,
    unlockNodeInteraction,
    unlockAllNodes,
  } = useLocker();

  useEffect(() => {
    if (!isOpen && selectedTab) {
      setSelectedTab(undefined);
    }
  }, [isOpen, selectedTab]);

  const isNodeLocked = (nodeId: string) =>
    interactionLockedNodes.some((lockedNode) => lockedNode.id === nodeId);

  const treeContent: TreeDataItem[] = [
    ...(nodes
      ?.filter((n) => n.type === "reader")
      .map((n) => ({
        id: n.id,
        name: n.data.name ?? "untitled",
        icon: isNodeLocked(n.id) ? Lock : Database,
      })) ?? []),
    ...(nodes
      ?.filter((n) => n.type === "writer")
      .map((n) => ({
        id: n.id,
        name: n.data.name ?? "untitled",
        icon: isNodeLocked(n.id) ? Lock : Disc,
      })) ?? []),
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
                name: n.data.name ?? "untitled",
                icon: isNodeLocked(n.id) ? Lock : Lightning,
              })),
          },
        ]
      : []),
    ...(nodes?.some((n) => n.type === "note")
      ? [
          {
            id: "note",
            name: t("Note"),
            icon: Note,
            children: nodes
              ?.filter((n) => n.type === "note")
              .map((n) => ({
                id: n.id,
                name: n.data.name ?? "untitled",
                icon: isNodeLocked(n.id) ? Lock : Note,
                style: isNodeLocked(n.id)
                  ? { color: "red", cursor: "not-allowed" }
                  : undefined, // 잠긴 노드는 회색 및 커서 변경
                children: {
                  id: n.id,
                  name: n.data.content ?? "untitled",
                },
              })),
          },
        ]
      : []),
    ...(nodes?.some((n) => n.type === "subworkflow")
      ? [
          {
            id: "subworkflow",
            name: t("Subworkflow"),
            icon: Plus,
            children: nodes
              ?.filter((n) => n.type === "subworkflow")
              .map((n) => ({
                id: n.id,
                name: n.data.name ?? "untitled",
                icon: isNodeLocked(n.id) ? Lock : Plus,
              })),
          },
        ]
      : []),
    ...(nodes?.some((n) => n.type === "batch")
      ? [
          {
            id: "batch",
            name: t("Batch Node"),
            icon: RectangleDashed,
            children: nodes
              ?.filter((n) => n.type === "batch")
              .map((n) => ({
                id: n.id,
                name: n.data.name ?? "untitled",
                icon: isNodeLocked(n.id) ? Lock : RectangleDashed,
                children: nodes
                  ?.filter((d) => d.parentId === n.id)
                  .map((d) => ({
                    id: d.id,
                    name: d.data.name ?? "untitled",
                    icon: getNodeIcon(d.type),
                  })),
              })),
          },
        ]
      : []),
  ];

  function getNodeIcon(type: string | undefined) {
    switch (type) {
      case "note":
        return Note;
      case "subworkflow":
        return Plus;
      case "transformer":
        return Lightning;
      case "reader":
        return Database;
      case "writer":
        return Disc;
      default:
        return Circle;
    }
  }

  var idContainer = "";

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
        <div className="flex flex-col">
          <Tree
            data={treeContent}
            className="w-full shrink-0 truncate rounded px-1"
            // initialSlelectedItemId="1"
            onSelectChange={(item) => {
              setContent(item?.name ?? "");

              if (typeof item?.id === "string") {
                idContainer = item.id;
              }
            }}
            onDoubleClick={() => {
              if (idContainer) {
                const node = nodes.find((n) => n.id === idContainer);
                if (node) {
                  if (interactionLockedNodes.some((n) => n.id === node.id)) {
                    unlockNodeInteraction(node);
                  } else {
                    lockNodeInteraction(node);
                  }
                }
              }
            }}
            // folderIcon={Folder}
            // itemIcon={Database}
          />
          <button
            onClick={unlockAllNodes}
            className="absolute bottom-2 right-2 w-24 h-8 rounded-lg bg-red-500 text-white text-xs flex items-center justify-center shadow-md hover:bg-red-400">
            Unlock All
          </button>
        </div>
      ),
    },
    {
      id: "actions-list",
      title: t("Actions list"),
      icon: <Lightning className="size-5" weight="thin" />,
      component: (
        <ActionsList
          nodes={nodes}
          onNodesChange={onNodesChange}
          onNodeLocking={onNodeLocking}
        />
      ),
    },
    {
      id: "resources",
      title: "Resources",
      icon: <HardDrive className="size-5" weight="thin" />,
      component: <Resources />,
    },
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
          {/* {content.title && <p>{content.title}</p>} */}
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
            <MagnifyingGlass
              className="size-6 cursor-pointer text-popover-foreground/50 hover:text-popover-foreground"
              weight="thin"
              onClick={() =>
                alert(
                  "Need to implement a global search and assign a shortcut as well",
                )
              }
            />
            <UserNavigation
              className="flex w-full justify-center"
              iconOnly
              dropdownPosition="right"
            />
            {/* <ProjectSettings
              className="flex items-center justify-center cursor-pointer rounded  transition-colors hover: md:h-8 md:w-8"
              dropdownPosition="right"
              dropdownOffset={15}
            /> */}
          </nav>
        </div>
      </aside>
    </>
  );
};

export default memo(LeftPanel);
