import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useT } from "@flow/lib/i18n";

type Props = {
  projectToBeDeleted: string | undefined;
  setProjectToBeDeleted: (project?: string) => void;
  onDeleteProject: (id: string) => void;
};

const ProjectDeletionDialog: React.FC<Props> = ({
  projectToBeDeleted,
  setProjectToBeDeleted,
  onDeleteProject,
}) => {
  const t = useT();
  return (
    <ConfirmationDialog
      title={t("Are you absolutely sure?")}
      description={t(
        "This action cannot be undone. This will permanently delete your project and remove your data from our servers.",
      )}
      isOpen={!!projectToBeDeleted}
      confirmDisabled={!projectToBeDeleted}
      onClose={() => setProjectToBeDeleted(undefined)}
      onConfirm={() =>
        projectToBeDeleted && onDeleteProject(projectToBeDeleted)
      }
    />
  );
};

export { ProjectDeletionDialog };
