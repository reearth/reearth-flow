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
  ContextMenuItem,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  disabled?: boolean;
  onProjectDelete: () => void;
};

const ProjectDeletionDialog: React.FC<Props> = ({
  disabled,
  onProjectDelete,
}) => {
  const t = useT();
  return (
    <AlertDialog>
      <AlertDialogTrigger asChild>
        <ContextMenuItem>{t("Delete Project")}</ContextMenuItem>
      </AlertDialogTrigger>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("Are you absolutely sure?")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t(
              `This action cannot be undone. 
              This will permanently delete your project and 
              remove your data from our servers.`
            )}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>Cancel</AlertDialogCancel>
          <AlertDialogAction onClick={onProjectDelete}>
            Continue
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
};

export default memo(ProjectDeletionDialog);
