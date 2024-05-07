import { FileIcon } from "@radix-ui/react-icons";
import { Database, Disc, Zap } from "lucide-react";
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
  const [isPanelOpen, handlePanelToggle] = useStateManager<boolean>(true);

  const [_content, setContent] = useState("Admin Page");

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
    {
      id: "transformer5",
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
    {
      id: "transformer3",
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
    {
      id: "transformer2",
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
    {
      id: "transformer4",
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
  ];

  const panelContents: PanelContent[] = [
    {
      id: "navigator",
      // title: t("Canvas"),
      icon: <FileIcon />,
      component: data && (
        <Tree
          data={treeContent}
          className="flex-shrink-0 w-full text-zinc-300 rounded truncate"
          // initialSlelectedItemId="1"
          onSelectChange={item => setContent(item?.name ?? "")}
          // folderIcon={Folder}
          // itemIcon={Database}
        />
      ),
    },
  ];
  return (
    <VerticalPanel
      className={`bg-zinc-800 border border-zinc-700 rounded-md backdrop-blur-md ${className}`}
      isOpen={!!isPanelOpen}
      headerContent={{
        id: "home-menu",
        icon: <FlowLogo />,
        component: <HomeMenu />,
      }}
      panelContents={panelContents}
      onPanelToggle={handlePanelToggle}
    />
  );
};

export default LeftPanel;
