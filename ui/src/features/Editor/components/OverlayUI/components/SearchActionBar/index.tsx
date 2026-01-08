import { MagnifyingGlassIcon } from "@phosphor-icons/react";
import { memo } from "react";

import {
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Workflow } from "@flow/types";

import { SearchPanel } from "../SearchPanel";

const tooltipOffset = 6;

type Props = {
  rawWorkflows: Workflow[];
  currentWorkflowId: string;
  onWorkflowOpen: (id: string) => void;
  showSearchPanel: boolean;
  onShowSearchPanel: (boolean: boolean) => void;
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
          <Popover
            open={showSearchPanel}
            onOpenChange={(open) => {
              if (!open) onShowSearchPanel(false);
            }}>
            <PopoverTrigger asChild>
              <IconButton
                className="rounded-[4px]"
                tooltipText={t("Search Actions")}
                tooltipOffset={tooltipOffset}
                tooltipPosition="left"
                onClick={() => onShowSearchPanel(true)}
                icon={<MagnifyingGlassIcon size={18} weight="light" />}
              />
            </PopoverTrigger>
            <PopoverContent
              onInteractOutside={(e) => e.preventDefault()}
              sideOffset={8}
              collisionPadding={5}
              className="h-[600px] w-100 bg-primary/50  backdrop-blur">
              {showSearchPanel && (
                <SearchPanel
                  rawWorkflows={rawWorkflows}
                  currentWorkflowId={currentWorkflowId}
                  onWorkflowOpen={onWorkflowOpen}
                  onShowSearchPanel={onShowSearchPanel}
                />
              )}
            </PopoverContent>
          </Popover>
        </div>
      </div>
    </div>
  );
};

export default memo(SearchActionBar);
