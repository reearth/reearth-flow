import ConfirmationDialog from "@flow/features/ConfirmationDialog";
import { useDocument } from "@flow/lib/gql/document";
import { useT } from "@flow/lib/i18n";
import { Project, ProjectDocument } from "@flow/types";

type Props = {
  duplicateProject: Project;
  setDuplicateProject: (project: Project | undefined) => void;
  onProjectDuplication: (
    project: Project,
    projectDocument?: ProjectDocument,
  ) => Promise<void>;
};

const ProjectDuplicateDialog: React.FC<Props> = ({
  duplicateProject,
  setDuplicateProject,
  onProjectDuplication,
}) => {
  const t = useT();
  const { useGetLatestProjectSnapshot } = useDocument();

  const { projectDocument } = useGetLatestProjectSnapshot(duplicateProject.id);

  const handleProjectDuplication = async () => {
    await onProjectDuplication(duplicateProject, projectDocument);
    setDuplicateProject(undefined);
  };

  return (
    <ConfirmationDialog
      title={t("Duplicate Project")}
      description={t(
        "This will duplicate {{project}} and all its contents. Are you sure you want to continue?",
        { project: duplicateProject.name },
      )}
      isOpen={!!duplicateProject}
      confirmDisabled={!duplicateProject}
      onClose={() => setDuplicateProject(undefined)}
      onConfirm={handleProjectDuplication}
    />
  );
};

export { ProjectDuplicateDialog };
