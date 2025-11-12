import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  onDebugRunStop: () => Promise<void>;
  onPopoverClose: () => void;
};

const DebugStopPopover: React.FC<Props> = ({
  onDebugRunStop,
  onPopoverClose,
}) => {
  const t = useT();

  const handleDebugRunStop = async () => {
    await onDebugRunStop();
    onPopoverClose();
  };

  return (
    <div className="flex flex-col">
      <div className="flex h-[61px] justify-between gap-2 p-4 ">
        <h4 className="text-md self-center rounded-t-lg leading-none tracking-tight dark:font-thin">
          {t("Stop Workflow")}
        </h4>
      </div>
      <div className="flex flex-col gap-2 px-4 pt-0">
        <p className="text-sm dark:font-light">
          {t("Are you sure you want to stop the workflow's debug run?")}
        </p>
        <div className="flex items-center justify-end pb-4">
          <Button variant="destructive" onClick={handleDebugRunStop}>
            {t("Stop")}
          </Button>
        </div>
      </div>
    </div>
  );
};

export default DebugStopPopover;
