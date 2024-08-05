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
          className="self-end"
        >
          {t("Delete Workspace")}
        </Button>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("Are you absolutely sure?")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t(
              `This action cannot be undone. 
              This will permanently delete your workspace and 
              remove your data from our servers.`
            )}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction onClick={onWorkspaceDelete}>
            Continue
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
};

export default memo(WorkspaceDeletionDialog);
