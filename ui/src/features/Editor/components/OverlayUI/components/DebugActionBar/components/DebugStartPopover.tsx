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
    <div className="flex flex-col gap-2">
      <div className="flex h-[61px] justify-between gap-2 border-b p-4 pb-2">
        <h4 className="text-md self-center rounded-t-lg leading-none tracking-tight dark:font-thin">
          {t("Start Debug Run")}
        </h4>
      </div>
      <div className="flex flex-col gap-4 px-4 pt-0">
        <p className="dark:font-light">
          {t("Are you sure you want to start a debug run of this workflow?")}
        </p>
        <div className="flex items-center justify-between gap-2 pb-2">
          <Button variant="outline" size="sm" onClick={onPopoverClose}>
            {t("Cancel")}
          </Button>
          <Button
            variant="default"
            size="sm"
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
