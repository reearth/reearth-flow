import { MagnifyingGlassIcon } from "@phosphor-icons/react";
import { memo, useState } from "react";

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

type PopoverOptions = "search" | undefined;

type Props = {
  rawWorkflows: Workflow[];
  currentWorkflowId: string;
  onWorkflowOpen: (id: string) => void;
};

const SearchActionBar: React.FC<Props> = ({
  rawWorkflows,
  currentWorkflowId,
  onWorkflowOpen,
}) => {
  const t = useT();
  const [showPopover, setShowPopover] = useState<PopoverOptions>(undefined);

  const handlePopoverOpen = (popover: PopoverOptions) =>
    setShowPopover(popover);
  const handlePopoverClose = () => setShowPopover(undefined);
  return (
    <div className="pointer-events-auto rounded-md p-1">
      <div className="flex rounded-md">
        <div className="flex flex-1 flex-col justify-end gap-1 align-middle">
          <Popover
            open={showPopover === "search"}
            onOpenChange={(open) => {
              if (!open) handlePopoverClose();
            }}>
            <PopoverTrigger asChild>
              <IconButton
                className="rounded-[4px]"
                tooltipText={t("Search Actions")}
                tooltipOffset={tooltipOffset}
                tooltipPosition="left"
                onClick={() => handlePopoverOpen("search")}
                icon={<MagnifyingGlassIcon size={18} weight="light" />}
              />
            </PopoverTrigger>
            <PopoverContent
              onInteractOutside={(e) => e.preventDefault()}
              sideOffset={8}
              collisionPadding={5}
              className="h-[600px] w-100 bg-primary/50  backdrop-blur">
              {showPopover === "search" && (
                <SearchPanel
                  rawWorkflows={rawWorkflows}
                  currentWorkflowId={currentWorkflowId}
                  onWorkflowOpen={onWorkflowOpen}
                  onPopoverClose={handlePopoverClose}
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
