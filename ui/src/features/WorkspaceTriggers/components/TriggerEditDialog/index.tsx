import {
  Button,
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  Label,
  DialogFooter,
  Select,
  SelectTrigger,
  SelectValue,
  SelectContent,
  SelectItem,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import { Trigger } from "@flow/types/trigger";

type Props = {
  selectedTrigger: Trigger;
  onDialogClose: () => void;
};

const TriggerEditDialog: React.FC<Props> = ({
  selectedTrigger,
  onDialogClose,
}) => {
  const t = useT();
  const eventSources = {
    cms: "Cms",
    api: "Api",
    manual: "Manual",
  };
  const sampleTimeIntervals = {
    EVERY_DAY: t("Every Day"),
    EVERY_HOUR: t("Every Hour"),
    EVERY_WEEK: t("Every Week"),
    EVERY_MONTH: t("Every Month"),
  };
  return (
    <Dialog open={true} onOpenChange={onDialogClose}>
      <DialogContent size="sm">
        <DialogTitle>{t("Edit Trigger")}</DialogTitle>
        <DialogContentWrapper>
          <div className="border-b border-primary text-center" />
          <DialogContentSection className="flex-1">
            <Label htmlFor="event-source-selector">
              {t("Select Event Source")}
            </Label>
            <Select>
              <SelectTrigger>
                <SelectValue placeholder={selectedTrigger.eventSource} />
              </SelectTrigger>
              <SelectContent>
                {Object.entries(eventSources).map(([value, label]) => (
                  <SelectItem key={value} value={value}>
                    {label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </DialogContentSection>
          <DialogContentSection className="flex-1">
            <Label htmlFor="time-interval-selector">
              {t("Select Time Interval")}
            </Label>
            <Select>
              <SelectTrigger>
                <SelectValue placeholder={sampleTimeIntervals.EVERY_DAY} />
              </SelectTrigger>
              <SelectContent>
                {Object.entries(sampleTimeIntervals).map(([value, label]) => (
                  <SelectItem key={value} value={value}>
                    {label}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          </DialogContentSection>
          <DialogContentSection>
            <p className="dark:font-light">
              {t("Are you sure you want to proceed?")}
            </p>
          </DialogContentSection>
        </DialogContentWrapper>
        <DialogFooter>
          <Button>{t("Update Trigger")}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { TriggerEditDialog };
