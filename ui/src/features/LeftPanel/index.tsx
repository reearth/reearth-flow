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
import { useT } from "@flow/providers";
import { useDialogType } from "@flow/stores";
import { Workflow } from "@flow/types";

import { UserNavigation } from "../Dashboard/components/Nav/components";

import { TransformerList, Resources, ProjectSettings } from "./components";

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
      icon: <TreeView className="h-5 w-5" weight="thin" />,
      component: data && (
        <Tree
          data={treeContent}
          className="flex-shrink-0 w-full px-1 text-zinc-300 rounded truncate"
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
      icon: <Lightning className="h-5 w-5" weight="thin" />,
      component: <TransformerList />,
    },
    {
      id: "resources",
      title: "Resources",
      icon: <HardDrive className="h-5 w-5" weight="thin" />,
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
        className="absolute left-12 z-10 flex flex-1 flex-col gap-3 h-full w-[300px] bg-zinc-900 border-r border-zinc-700 transition-all overflow-auto"
        style={{
          transform: `translateX(${isPanelOpen ? "8px" : "-100%"})`,
          transitionDuration: isPanelOpen ? "500ms" : "300ms",
          transitionProperty: "transform",
          transitionTimingFunction: "cubic-bezier(0.4, 0, 0.2, 1)",
        }}>
        <div className="flex flex-col gap-2 px-4 py-2 border-b border-zinc-700/50">
          <p className="text-lg font-thin">{tabs?.find(tc => tc.id === selectedTab)?.title}</p>
        </div>
        <div className="flex flex-col gap-2 overflow-auto">
          {/* {content.title && <p className="text-md">{content.title}</p>} */}
          {tabs?.find(tc => tc.id === selectedTab)?.component}
        </div>
      </div>
      <aside className="relative w-14 z-10  border-r border-zinc-700 bg-zinc-800">
        <div className="bg-zinc-900/50 h-full flex flex-col">
          <nav className="flex flex-col items-center gap-4 p-2">
            <Link
              to={`/workspace/${workspaceId}`}
              className="flex p-2 shrink-0 items-center justify-center gap-2 rounded bg-red-800/50 text-lg font-semibold text-primary-foreground md:h-8 md:w-8 md:text-base hover:bg-red-800/80">
              <FlowLogo className="h-5 w-5" />
              <span className="sr-only">{t("Dashboard")}</span>
            </Link>
            {tabs.map(tab => (
              <IconButton
                key={tab.id}
                className={`flex h-9 w-9 items-center justify-center rounded text-zinc-500 transition-colors hover:text-zinc-300 md:h-8 md:w-8 ${selectedTab === tab.id && "bg-zinc-700/80 text-zinc-300"}`}
                icon={tab.icon}
                onClick={() => handleTabChange(tab.id)}
              />
            ))}
          </nav>
          <nav className="mt-auto flex flex-col items-center gap-4 px-2 py-2">
            <MagnifyingGlass
              className="h-6 w-6 text-zinc-400 cursor-pointer hover:text-zinc-300"
              weight="thin"
              onClick={() => setDialogType("canvas-search")}
            />
            <UserNavigation
              className="w-full flex justify-center"
              iconOnly
              dropdownPosition="right"
            />
            <ProjectSettings
              className="flex items-center justify-center cursor-pointer rounded text-zinc-400 transition-colors hover:text-zinc-300 md:h-8 md:w-8"
              dropdownPosition="right"
              dropdownOffset={15}
            />
          </nav>
        </div>
      </aside>
    </>
  );
};

export default LeftPanel;
