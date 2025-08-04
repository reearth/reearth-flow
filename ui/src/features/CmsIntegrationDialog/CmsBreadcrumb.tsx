import { LayoutIcon } from "@phosphor-icons/react";

import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { CmsProject, CmsModel, CmsItem } from "@flow/types/cmsIntegration";

type ViewMode = "projects" | "models" | "items" | "itemDetails" | "itemsAssets";

type Props = {
  viewMode: ViewMode;
  selectedProject?: CmsProject | null;
  selectedModel?: CmsModel | null;
  selectedItem?: CmsItem | null;
  onBackToProjects: () => void;
  onBackToModels: () => void;
  onBackToItems: () => void;
};

const CmsBreadcrumb: React.FC<Props> = ({
  viewMode,
  selectedProject,
  selectedModel,
  selectedItem,
  onBackToProjects,
  onBackToModels,
  onBackToItems,
}) => {
  const t = useT();

  return (
    <div className="flex items-center font-normal">
      <LayoutIcon size={24} className="mr-2 inline-block" />
      <span className="px-4 py-2 pr-1 pl-1">{t("CMS Integration")}</span>

      {selectedProject && (
        <div>
          <span className="mx-2 text-muted-foreground">/</span>
          <Button
            variant="ghost"
            onClick={onBackToProjects}
            className="text-md pr-1 pl-1 font-normal dark:font-thin">
            {selectedProject.name}
          </Button>
        </div>
      )}

      {selectedProject && selectedModel && (
        <div>
          <span className="mx-2 text-muted-foreground">/</span>
          <Button
            variant="ghost"
            onClick={onBackToModels}
            className="text-md pr-1 pl-1 font-normal dark:font-thin">
            {selectedModel.name}
          </Button>
        </div>
      )}

      {selectedItem && (
        <div>
          <span className="mx-2 text-muted-foreground">/</span>
          <span className="px-4 py-2 pr-1 pl-1">
            <Button
              variant="ghost"
              onClick={onBackToItems}
              className="text-md pr-1 pl-1 font-normal dark:font-thin">
              {selectedItem.id}
            </Button>
            <span className="mx-2 text-muted-foreground">/</span>
            {viewMode === "itemsAssets" && (
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
