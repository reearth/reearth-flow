import { Button } from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  debugRunStarted?: boolean;
  onDebugRunStart: () => Promise<void>;
  onPopoverClose: () => void;
};

const DebugStartPopover: React.FC<Props> = ({
  debugRunStarted,
  onDebugRunStart,
  onPopoverClose,
}) => {
  const t = useT();

  const handleDebugRunStart = async () => {
    await onDebugRunStart();
    onPopoverClose();
  };

  return (
    <div className="flex flex-col">
      <div className="flex h-[61px] justify-between gap-2 p-4">
        <h4 className="text-md self-center rounded-t-lg leading-none tracking-tight dark:font-thin">
          {t("Start Debug Run")}
        </h4>
      </div>
      <div className="flex flex-col gap-2 px-4 pt-0">
        <p className="text-sm dark:font-light">
          {t("Are you sure you want to start a debug run of this workflow?")}
        </p>
        <div className="flex items-center justify-end pb-4">
          <Button
            variant="outline"
            onClick={handleDebugRunStart}
            disabled={debugRunStarted}>
            {t("Start")}
          </Button>
        </div>
      </div>
    </div>
  );
};

export default DebugStartPopover;
