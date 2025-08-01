import { EyeIcon, DatabaseIcon } from "@phosphor-icons/react";

import {
  Button,
  Card,
  CardContent,
  CardDescription,
  CardHeader,
  CardTitle,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { CmsModel } from "@flow/types/cmsIntegration";

type Props = {
  model: CmsModel;
  onModelSelect: (model: CmsModel) => void;
};

const CmsModelCard: React.FC<Props> = ({ model, onModelSelect }) => {
  const t = useT();

  return (
    <Card
      key={model.id}
      className="cursor-pointer transition-shadow hover:shadow-md">
      <CardHeader className="p-2">
        <div className="flex items-start justify-between">
          <div className="flex-1">
            <CardTitle className="flex items-center gap-2 text-base">
              <DatabaseIcon size={16} />
              {model.name}
            </CardTitle>
            <CardDescription className="text-sm text-muted-foreground">
              {model.description}
            </CardDescription>
          </div>
        </div>
      </CardHeader>
      <CardContent className="p-2">
        <div className="flex items-center justify-between text-xs text-muted-foreground">
          <div className="text-xs text-muted-foreground">
            {model.schema.fields.length} fields
          </div>
          <span className="flex items-center gap-1">
            {t("Updated At")} {model.updatedAt}
          </span>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 px-2"
            onClick={(e) => {
              e.stopPropagation();
              onModelSelect(model);
            }}>
            <EyeIcon size={14} />
            {t("View Items")}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
};

export default CmsModelCard;
