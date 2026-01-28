import { TrashIcon } from "@phosphor-icons/react";
import { memo } from "react";

import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
  Button,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  disabled?: boolean;
  onWorkspaceDelete: () => void;
};

const WorkspaceDeletionDialog: React.FC<Props> = ({
  disabled,
  onWorkspaceDelete,
}) => {
  const t = useT();
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <Button
          variant={"destructive"}
          disabled={disabled}
          size="sm"
          className="self-end">
          <TrashIcon />
          {t("Delete")}
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("Are you absolutely sure?")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t(
              "This action cannot be undone. This will permanently delete your workspace and remove your data from our servers.",
            )}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t("Cancel")}</AlertDialogCancel>
          <AlertDialogAction onClick={onWorkspaceDelete}>
            {t("Continue")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
};

export default memo(WorkspaceDeletionDialog);
