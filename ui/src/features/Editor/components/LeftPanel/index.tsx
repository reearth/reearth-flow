import {
  Database,
  Disc,
  HardDrive,
  Lightning,
  TreeView,
} from "@phosphor-icons/react";
import { Link, useParams } from "@tanstack/react-router";
import { memo, useEffect, useState } from "react";

import { FlowLogo, Tree, TreeDataItem, IconButton } from "@flow/components";
import { UserMenu } from "@flow/features/common";
import { useShortcuts } from "@flow/hooks";
import { useT } from "@flow/lib/i18n";
import type { Node } from "@flow/types";

import { ActionsList, Resources } from "./components";

type Tab = "navigator" | "actions-list" | "resources";

type Props = {
  nodes: Node[];
  isOpen: boolean;
  onOpen: (panel?: "left" | "right" | "bottom") => void;
  onNodesAdd: (node: Node[]) => void;
  isMainWorkflow: boolean;
  hasReader?: boolean;
};

const LeftPanel: React.FC<Props> = ({
  nodes,
  isOpen,
  onOpen,
  onNodesAdd,
  isMainWorkflow,
  hasReader,
}) => {
  const t = useT();
  const { workspaceId } = useParams({ strict: false });
  const [selectedTab, setSelectedTab] = useState<Tab | undefined>();

  const [_content, setContent] = useState("Admin Page");

  useEffect(() => {
    if (!isOpen && selectedTab) {
      setSelectedTab(undefined);
    }
  }, [isOpen, selectedTab]);

  const treeContent: TreeDataItem[] = [
    ...(nodes
      ?.filter((n) => n.type === "reader")
      .map((n) => ({
        id: n.id,
        name: n.data.customName || n.data.officialName || "untitled",
        icon: Database,
      })) ?? []),
    ...(nodes
      ?.filter((n) => n.type === "writer")
      .map((n) => ({
        id: n.id,
        name: n.data.customName || n.data.officialName || "untitled",
        icon: Disc,
      })) ?? []),
    {
      id: "transformer",
      name: t("Transformers"),
      icon: Lightning,
      children: nodes
        ?.filter((n) => n.type === "transformer")
        .map((n) => ({
          id: n.id,
          name: n.data.customName || n.data.officialName || "untitled",
          // icon: Disc,
        })),
    },
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
          // initialSlelectedItemId="1"
          onSelectChange={(item) => setContent(item?.name ?? "")}
          // folderIcon={Folder}
          // itemIcon={Database}
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
