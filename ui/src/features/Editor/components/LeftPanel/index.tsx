import {
  Database,
  Disc,
  HardDrive,
  Lightning,
  MagnifyingGlass,
  TreeView,
} from "@phosphor-icons/react";
import { Link, useParams } from "@tanstack/react-router";
import { useState } from "react";

import { FlowLogo, Tree, TreeDataItem, IconButton } from "@flow/components";
import { UserNavigation } from "@flow/features/TopNavigation/components";
import { useT } from "@flow/lib/i18n";
import { useDialogType } from "@flow/stores";
import { Workflow } from "@flow/types";

import { TransformerList, Resources } from "./components";

type Tab = "navigator" | "transformer-list" | "resources";

type Props = {
  data?: Workflow;
};

const LeftPanel: React.FC<Props> = ({ data }) => {
  const t = useT();
  const { workspaceId } = useParams({ strict: false });
  const [isPanelOpen, setPanelOpen] = useState(false);
  const [selectedTab, setSelectedTab] = useState<Tab>("navigator");

  const [_content, setContent] = useState("Admin Page");

  const [, setDialogType] = useDialogType();

  const treeContent: TreeDataItem[] = [
    ...(data?.nodes
      ?.filter(n => n.type === "reader")
      .map(n => ({
        id: n.id,
        name: n.data.name ?? "untitled",
        icon: Database,
      })) ?? []),
    ...(data?.nodes
      ?.filter(n => n.type === "writer")
      .map(n => ({
        id: n.id,
        name: n.data.name ?? "untitled",
        icon: Disc,
      })) ?? []),
    {
      id: "transformer",
      name: t("Transformers"),
      icon: Lightning,
      children: data?.nodes
        ?.filter(n => n.type === "transformer")
        .map(n => ({
          id: n.id,
          name: n.data.name ?? "untitled",
          // icon: Disc,
        })),
    },
  ];

  const tabs: { id: Tab; title: string; icon: React.ReactNode; component: React.ReactNode }[] = [
    {
      id: "navigator",
      title: t("Canvas Navigation"),
      icon: <TreeView className="size-5" weight="thin" />,
      component: data && (
        <Tree
          data={treeContent}
          className="w-full shrink-0 truncate rounded px-1 text-zinc-300"
          // initialSlelectedItemId="1"
          onSelectChange={item => setContent(item?.name ?? "")}
          // folderIcon={Folder}
          // itemIcon={Database}
        />
      ),
    },
    {
      id: "transformer-list",
      title: t("Transformer list"),
      icon: <Lightning className="size-5" weight="thin" />,
      component: <TransformerList />,
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
      setPanelOpen(!isPanelOpen);
    } else {
      setSelectedTab(tab);
      if (!isPanelOpen) {
        setPanelOpen(true);
      }
    }
  };

  return (
    <>
      <div
        className="absolute left-12 z-10 flex h-full w-[300px] flex-1 flex-col gap-3 overflow-auto border-r border-zinc-700 bg-zinc-900 transition-all"
        style={{
          transform: `translateX(${isPanelOpen ? "8px" : "-100%"})`,
          transitionDuration: isPanelOpen ? "500ms" : "300ms",
          transitionProperty: "transform",
          transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
        }}>
        <div className="flex flex-col gap-2 border-b border-zinc-700/50 px-4 py-2">
          <p className="text-lg font-thin">{tabs?.find(tc => tc.id === selectedTab)?.title}</p>
        </div>
        <div className="flex flex-col gap-2 overflow-auto">
          {/* {content.title && <p>{content.title}</p>} */}
          {tabs?.find(tc => tc.id === selectedTab)?.component}
        </div>
      </div>
      <aside className="relative z-10 w-14  border-r border-zinc-700 bg-background-800">
        <div className="flex h-full flex-col bg-zinc-900/50">
          <nav className="flex flex-col items-center gap-4 p-2">
            <Link
              to={`/workspace/${workspaceId}`}
              className="flex shrink-0 items-center justify-center gap-2 rounded bg-red-800/50 p-2 text-lg font-semibold text-primary-foreground hover:bg-red-800/80 md:size-8 md:text-base">
              <FlowLogo className="size-5" />
              <span className="sr-only">{t("Dashboard")}</span>
            </Link>
            {tabs.map(tab => (
              <IconButton
                key={tab.id}
                className={`flex size-9 items-center justify-center rounded text-zinc-500 transition-colors hover:text-zinc-300 md:size-8 ${selectedTab === tab.id && "bg-background-700/80 text-zinc-300"}`}
                icon={tab.icon}
                onClick={() => handleTabChange(tab.id)}
              />
            ))}
          </nav>
          <nav className="mt-auto flex flex-col items-center gap-4 p-2">
            <MagnifyingGlass
              className="size-6 cursor-pointer text-zinc-400 hover:text-zinc-300"
              weight="thin"
              onClick={() => setDialogType("canvas-search")}
            />
            <UserNavigation
              className="flex w-full justify-center"
              iconOnly
              dropdownPosition="right"
            />
            {/* <ProjectSettings
              className="flex items-center justify-center cursor-pointer rounded text-zinc-400 transition-colors hover:text-zinc-300 md:h-8 md:w-8"
              dropdownPosition="right"
              dropdownOffset={15}
            /> */}
          </nav>
        </div>
      </aside>
    </>
  );
};

export { LeftPanel };
