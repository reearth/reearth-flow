import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  projectToBeDeleted: string | undefined;
  setProjectToBeDeleted: (project: string | undefined) => void;
  onDeleteProject: (id: string) => void;
};

const ProjectDeletionDialog: React.FC<Props> = ({
  projectToBeDeleted,
  setProjectToBeDeleted,
  onDeleteProject,
}) => {
  const t = useT();
  return (
    <AlertDialog open={!!projectToBeDeleted}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t("Are you absolutely sure?")}</AlertDialogTitle>
          <AlertDialogDescription>
            {t(
              "This action cannot be undone. This will permanently delete your project and remove your data from our servers.",
            )}
          </AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel onClick={() => setProjectToBeDeleted(undefined)}>
            {t("Cancel")}
          </AlertDialogCancel>
          <AlertDialogAction
            disabled={!projectToBeDeleted}
            onClick={() =>
              projectToBeDeleted && onDeleteProject(projectToBeDeleted)
            }>
            {t("Continue")}
          </AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
  );
};

export { ProjectDeletionDialog };
