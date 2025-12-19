import { PlayIcon } from "@phosphor-icons/react";

import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { AnyWorkflowVariable } from "@flow/types";

type Props = {
  debugRunStarted?: boolean;
  customDebugRunWorkflowVariables?: AnyWorkflowVariable[];
  onDebugRunStart: () => Promise<void>;
  onShowDebugWorkflowVariablesDialog: () => void;
  onPopoverClose: () => void;
};

const DebugStartPopover: React.FC<Props> = ({
  debugRunStarted,
  onDebugRunStart,
  customDebugRunWorkflowVariables,
  onShowDebugWorkflowVariablesDialog,
  onPopoverClose,
}) => {
  const t = useT();

  const handleDebugRunStart = async () => {
    await onDebugRunStart();
    onPopoverClose();
  };

  return (
    <div className="flex flex-col gap-2 p-4">
      <div className="flex justify-between gap-2">
        <h4 className="text-md flex items-center gap-2 self-center rounded-t-lg leading-none tracking-tight dark:font-thin">
          <PlayIcon weight="thin" size={18} />
          {t("Start Debug Run")}
        </h4>
      </div>
      <div className="flex flex-col gap-2">
        <p className="text-sm dark:font-light">
          {t("Are you sure you want to start a debug run of this workflow?")}
        </p>
        <div className="flex items-center justify-end">
          <Button
            variant="outline"
            onClick={
              customDebugRunWorkflowVariables?.length === 0
                ? handleDebugRunStart
                : onShowDebugWorkflowVariablesDialog
            }
            disabled={debugRunStarted}>
            {t("Start")}
          </Button>
        </div>
      </div>
    </div>
  );
};

export default DebugStartPopover;
