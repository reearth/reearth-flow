import { MagnifyingGlassIcon } from "@phosphor-icons/react";
import { memo } from "react";

import { IconButton } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Workflow } from "@flow/types";

import { SearchPanel } from "./components";

const tooltipOffset = 6;

type Props = {
  rawWorkflows: Workflow[];
  currentWorkflowId: string;
  onWorkflowOpen: (id: string) => void;
  showSearchPanel: boolean;
  onShowSearchPanel: (open: boolean) => void;
};

const SearchActionBar: React.FC<Props> = ({
  rawWorkflows,
  currentWorkflowId,
  onWorkflowOpen,
  showSearchPanel,
  onShowSearchPanel,
}) => {
  const t = useT();
  return (
    <div className="pointer-events-auto rounded-md p-1">
      <div className="flex rounded-md">
        <div className="flex flex-1 flex-col justify-end gap-1 align-middle">
          <div className={showSearchPanel ? "invisible" : ""}>
            <IconButton
              className="rounded-[4px]"
              tooltipText={t("Search Canvas")}
              tooltipOffset={tooltipOffset}
              tooltipPosition="left"
              onClick={() => onShowSearchPanel(true)}
              icon={<MagnifyingGlassIcon size={18} weight="light" />}
            />
          </div>
          {showSearchPanel && (
            <SearchPanel
              rawWorkflows={rawWorkflows}
              currentWorkflowId={currentWorkflowId}
              onWorkflowOpen={onWorkflowOpen}
              onShowSearchPanel={onShowSearchPanel}
            />
          )}
        </div>
      </div>
    </div>
  );
};

export default memo(SearchActionBar);
