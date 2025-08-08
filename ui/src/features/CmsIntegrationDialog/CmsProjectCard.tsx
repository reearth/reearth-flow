import { EyeIcon, GlobeIcon } from "@phosphor-icons/react";
import { LockIcon } from "@phosphor-icons/react/dist/ssr";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { CmsProject } from "@flow/types/cmsIntegration";

type Props = {
  project: CmsProject;
  onProjectSelect: (project: CmsProject) => void;
};

const CmsProjectCard: React.FC<Props> = ({ project, onProjectSelect }) => {
  const t = useT();

  return (
    <Card
      className="flex cursor-pointer flex-col justify-between transition-shadow hover:shadow-md"
      onDoubleClick={() => onProjectSelect(project)}>
      <CardHeader className="p-2">
        <div className="flex items-start justify-between">
          <div className="min-w-0 flex-1 ">
            <CardTitle className="flex items-center gap-2 truncate text-base">
              <span className="truncate">{project.name}</span>
              {project.visibility === "public" ? (
                <span className="flex items-center gap-1 rounded-full bg-logo/20 px-2 py-0.5 text-xs text-green-500">
                  <GlobeIcon size={14} />
                  {t("Public")}
                </span>
              ) : (
                <span className="flex items-center gap-1 rounded-full bg-destructive/20 px-2 py-0.5 text-xs text-orange-700">
                  <LockIcon size={14} />
                  {t("Private")}
                </span>
              )}
            </CardTitle>
            <CardDescription className="text-sm text-muted-foreground">
              @{project.alias}
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="p-2">
        {project.description && (
          <p className="mb-3 line-clamp-2 truncate text-sm text-muted-foreground">
            {project.description}
          </p>
        )}
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <span className="flex items-center gap-1">
            {t("Updated At")} {""}
            {new Date(project.updatedAt).toLocaleDateString()}
          </span>
          <Button
            variant="ghost"
            size="sm"
            disabled={project.visibility === "private"}
            className="h-7 px-2"
            onClick={(e) => {
              e.stopPropagation();
              onProjectSelect(project);
            }}>
            <EyeIcon size={14} />
            {t("View Models")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
};

export default CmsProjectCard;
