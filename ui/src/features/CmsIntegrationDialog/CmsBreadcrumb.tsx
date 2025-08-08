import { LayoutIcon } from "@phosphor-icons/react";

import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { CmsProject, CmsModel, CmsItem } from "@flow/types/cmsIntegration";

import { ViewMode } from "./hooks";

type Props = {
  viewMode: ViewMode;
  selectedProject?: CmsProject | null;
  selectedModel?: CmsModel | null;
  selectedItem?: CmsItem | null;
  onBackToModels: () => void;
  onBackToItems: () => void;
};

const CmsBreadcrumb: React.FC<Props> = ({
  viewMode,
  selectedProject,
  selectedModel,
  selectedItem,
  onBackToModels,
  onBackToItems,
}) => {
  const t = useT();

  return (
    <div className="flex min-w-0 items-center font-normal">
      <LayoutIcon size={24} className="mr-2 shrink-0" />
      <span className="shrink-0 px-4 py-2 pr-1 pl-1">
        {t("CMS Integration")}
      </span>

      {selectedProject && (
        <div className="flex min-w-0 items-center">
          <span className="mx-2 shrink-0 text-muted-foreground">/</span>
          <Button
            variant="ghost"
            onClick={onBackToModels}
            className="text-md max-w-full min-w-0 truncate overflow-hidden pr-1 pl-1 font-normal dark:font-thin">
            <span className="truncate">{selectedProject.name}</span>
          </Button>
        </div>
      )}

      {selectedProject && selectedModel && (
        <div className="flex min-w-0 items-center">
          <span className="mx-2 shrink-0 text-muted-foreground">/</span>
          <Button
            variant="ghost"
            onClick={onBackToItems}
            className="text-md max-w-full min-w-0 truncate overflow-hidden pr-1 pl-1 font-normal dark:font-thin">
            <span className="truncate">{selectedModel.name}</span>
          </Button>
        </div>
      )}
      {selectedItem && (
        <div>
          <span className="mx-2 text-muted-foreground">/</span>
          <span className="px-4 py-2 pr-1 pl-1">
            <span className="text-md pr-1 pl-1 font-normal dark:font-thin">
              {selectedItem.id}
            </span>
            <span className="mx-2 text-muted-foreground">/</span>
            {viewMode === "itemAssets" && (
              <span className="dark:font-thin">{t("Assets")}</span>
            )}
            {viewMode === "itemDetails" && (
              <span className="dark:font-thin">{t("Details")}</span>
            )}
          </span>
        </div>
      )}
    </div>
  );
};

export default CmsBreadcrumb;
