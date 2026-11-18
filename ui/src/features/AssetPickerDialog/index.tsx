import { useState } from "react";

import {
  Button,
  CmsLogo,
  Dialog,
  DialogContent,
  DialogContentWrapper,
  DialogFooter,
  DialogTitle,
  FlowLogo,
  Tabs,
  TabsContent,
  TabsList,
  TabsTrigger,
} from "@flow/components";
import { WorkspaceAssetsPanel } from "@flow/features/AssetsDialog";
import { useT } from "@flow/lib/i18n";
import { useCurrentWorkspace } from "@flow/stores";

import CmsAssetsPanel from "./components/CmsAssetsPanel";
import CmsModelsPanel from "./components/CmsModelsPanel";

export type AssetPickerTab = "flowAssets" | "cmsAssets" | "cmsModels";

type Selection = { tab: AssetPickerTab; id: string; url: string };

type Props = {
  onDialogClose: () => void;
  onSelect: (url: string) => void;
};

const AssetPickerDialog: React.FC<Props> = ({ onDialogClose, onSelect }) => {
  const t = useT();
  const [currentWorkspace] = useCurrentWorkspace();
  const workspaceId = currentWorkspace?.id ?? "";

  const [tab, setTab] = useState<AssetPickerTab>("flowAssets");
  const [selection, setSelection] = useState<Selection>();

  const activeSelection = selection?.tab === tab ? selection : undefined;

  const select = (url: string) => {
    onSelect(url);
    onDialogClose();
  };

  const handleClick =
    (selectedTab: AssetPickerTab) => (item: { id: string; url: string }) =>
      setSelection({ tab: selectedTab, id: item.id, url: item.url });

  const handleDoubleClick = (item: { url: string }) => select(item.url);

  return (
    <Dialog open onOpenChange={onDialogClose}>
      <DialogContent className="max-h-[90vh] w-full max-w-5xl overflow-hidden">
        <Tabs
          value={tab}
          onValueChange={(v) => setTab(v as AssetPickerTab)}
          className="flex flex-col">
          <DialogTitle className="flex items-center">
            <TabsList className="h-auto gap-1">
              <TabsTrigger value="flowAssets" className="gap-1.5">
                <FlowLogo className="size-4" />
                {t("Flow Workspace Assets")}
              </TabsTrigger>
              <TabsTrigger value="cmsAssets" className="gap-1.5">
                <CmsLogo className="size-4 text-[#F7B502]" />
                {t("CMS Assets")}
              </TabsTrigger>
              <TabsTrigger value="cmsModels" className="gap-1.5">
                <CmsLogo className="size-4 text-[#F7B502]" />
                {t("CMS Models")}
              </TabsTrigger>
            </TabsList>
          </DialogTitle>
          <DialogContentWrapper className="pb-0">
            <TabsContent value="flowAssets">
              <WorkspaceAssetsPanel
                workspaceId={workspaceId}
                selectedAssetId={activeSelection?.id}
                onAssetClick={handleClick("flowAssets")}
                onAssetDoubleClick={handleDoubleClick}
              />
            </TabsContent>
            <TabsContent value="cmsAssets">
              <CmsAssetsPanel
                workspaceId={workspaceId}
                selectedAssetId={activeSelection?.id}
                onAssetClick={handleClick("cmsAssets")}
                onAssetDoubleClick={handleDoubleClick}
              />
            </TabsContent>
            <TabsContent value="cmsModels">
              <CmsModelsPanel workspaceId={workspaceId} onSelect={select} />
            </TabsContent>
          </DialogContentWrapper>
          <DialogFooter>
            <Button variant="outline" onClick={onDialogClose}>
              {t("Cancel")}
            </Button>
            <Button
              disabled={!activeSelection}
              onClick={() => activeSelection && select(activeSelection.url)}>
              {t("Select")}
            </Button>
          </DialogFooter>
        </Tabs>
      </DialogContent>
    </Dialog>
  );
};

export default AssetPickerDialog;
