import { CopyIcon } from "@phosphor-icons/react";
import { useCallback } from "react";

import {
  Button,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogFooter,
  DialogTitle,
  IconButton,
} from "@flow/components";
import { useToast } from "@flow/features/NotificationSystem/useToast";
import { useT } from "@flow/lib/i18n";
import { copyToClipboard } from "@flow/utils/copyToClipboard";

// A token the server just issued. Only its hint is kept afterwards, so this is
// the one time it can be read.
const TriggerAuthTokenReveal: React.FC<{ authToken: string }> = ({
  authToken,
}) => {
  const t = useT();
  const { toast } = useToast();

  const handleCopy = useCallback(() => {
    copyToClipboard(authToken);
    toast({
      title: t("Copied to clipboard"),
      description: t("Auth token copied to clipboard"),
    });
  }, [authToken, t, toast]);

  return (
    <div className="flex flex-col gap-2 text-sm">
      <span className="font-semibold">{t("Auth Token")}</span>
      <div className="flex items-center gap-2">
        <span className="min-w-0 flex-1 overflow-x-auto rounded border bg-background px-2 py-1 font-mono text-xs whitespace-nowrap">
          {authToken}
        </span>
        <IconButton
          size="icon"
          variant="ghost"
          className="shrink-0"
          onClick={handleCopy}
          icon={<CopyIcon />}
        />
      </div>
      <p className="text-xs text-muted-foreground">
        {t("This is your secret token, please use as your env value.")}
      </p>
      <p className="text-xs text-warning">
        {t("Copy it now. It won't be shown again.")}
      </p>
    </div>
  );
};

type NewAuthTokenProps = {
  authToken: string;
  onDone: () => void;
};

// The dialog content for a token issued by an update or a re-generation.
const TriggerNewAuthToken: React.FC<NewAuthTokenProps> = ({
  authToken,
  onDone,
}) => {
  const t = useT();
  return (
    <DialogContent size="sm">
      <DialogTitle>{t("New Auth Token")}</DialogTitle>
      <DialogContentWrapper>
        <DialogContentSection>
          <TriggerAuthTokenReveal authToken={authToken} />
        </DialogContentSection>
      </DialogContentWrapper>
      <DialogFooter>
        <Button onClick={onDone}>{t("Done")}</Button>
      </DialogFooter>
    </DialogContent>
  );
};

export { TriggerAuthTokenReveal, TriggerNewAuthToken };
