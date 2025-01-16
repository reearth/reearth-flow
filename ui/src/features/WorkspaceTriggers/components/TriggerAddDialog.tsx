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

type Props = {
  setShowDialog: (show: boolean) => void;
};

const TriggerAddDialog: React.FC<Props> = ({ setShowDialog }) => {
  const t = useT();

  const sampleEventSources = {
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
    <Dialog open={true} onOpenChange={() => setShowDialog(false)}>
      <DialogContent size="sm">
        <DialogTitle>{t("Create a new trigger")}</DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="flex-1">
            <Label htmlFor="event-source-selector">
              {t("Select Event Source")}
            </Label>
            <Select>
              <SelectTrigger>
                <SelectValue placeholder={sampleEventSources.manual} />
              </SelectTrigger>
              <SelectContent>
                {Object.entries(sampleEventSources).map(([value, label]) => (
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
          <Button>{t("Add New Trigger")}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
};

export { TriggerAddDialog };
