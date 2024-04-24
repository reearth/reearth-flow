import { FileIcon } from "@radix-ui/react-icons";
import { Database, Disc, SearchIcon, Zap } from "lucide-react";
import { useState } from "react";

import { VerticalPanel, FlowLogo, type PanelContent, Tree, TreeDataItem } from "@flow/components";
import { useStateManager } from "@flow/hooks";
import { Workflow } from "@flow/types";

import HomeMenu from "./components/HomeMenu";

type Props = {
  className?: string;
  data?: Workflow;
};

const LeftPanel: React.FC<Props> = ({ className, data }) => {
  const [searchText, setSearchText] = useState<string>("");
  const [isPanelOpen, handlePanelToggle] = useStateManager<boolean>(true);

  const [_content, setContent] = useState("Admin Page");
  console.log("data: ", data);

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
      name: "Transformers",
      icon: Zap,
      children: data?.nodes
        ?.filter(n => n.type === "transformer")
        .map(n => ({
          id: n.id,
          name: n.data.name ?? "untitled",
          // icon: Disc,
        })),
    },
    // {
    //   id: "batch",
    //   name: "Batches",
    //   children: data?.nodes
    //     ?.filter(n => n.type === "batch")
    //     .map(n => ({
    //       id: n.id,
    //       name: n.data.name ?? "untitled",
    //       // icon: Group,
    //     })),
    // },
    // {
    //   id: "notes",
    //   name: "Notes",
    //   children: data?.nodes
    //     ?.filter(n => n.type === "note")
    //     .map(n => ({
    //       id: n.id,
    //       name: n.data.name ?? "untitled",
    //       // icon: Pen,
    //     })),
    // },
  ];

  const panelContents: PanelContent[] = [
    {
      id: "home-menu",
      icon: <FlowLogo />,
      component: (
        <>
          <HomeMenu />
          {/* <div className="border-zinc-700 border-t-[1px] w-[100%]" /> */}
        </>
      ),
    },
    {
      id: "navigator",
      // title: t("Navigator"),
      icon: <FileIcon />,
      component: (
        <>
          <div className="flex gap-2 items-center bg-zinc-700/50 rounded-sm px-2 py-1 placeholder-zinc-300/40 text-sm">
            {searchText.length < 1 && <SearchIcon className="w-4 h-4 text-zinc-400" />}
            <input
              className="bg-transparent w-full text-zinc-300 placeholder-zinc-500"
              placeholder="Search data"
              value={searchText}
              onChange={s => setSearchText(s.target.value)}
            />
          </div>
          <div className="border-zinc-700/50 border-t-[1px] w-[100%]" />
          {data && (
            <Tree
              data={treeContent}
              className="flex-shrink-0 w-full h-[60vh] text-zinc-300"
              // initialSlelectedItemId="1"
              onSelectChange={item => setContent(item?.name ?? "")}
              // folderIcon={Folder}
              // itemIcon={Database}
            />
          )}
          {/* <div className="border-zinc-700 border-t-[1px] w-[100%]" /> */}
        </>
      ),
    },
  ];
  return (
    <VerticalPanel
      className={`bg-zinc-800 bg-opacity-75 rounded-md backdrop-blur-md ${className}`}
      isOpen={!!isPanelOpen}
      togglePosition="end-right"
      panelContents={panelContents}
      onPanelToggle={handlePanelToggle}
    />
  );
};

export default LeftPanel;
