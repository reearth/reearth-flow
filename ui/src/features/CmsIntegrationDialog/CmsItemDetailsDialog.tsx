import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
  ScrollArea,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { CmsItem, CmsModel } from "@flow/types/cmsIntegration";

type Props = {
  cmsItem: CmsItem;
  cmsModel: CmsModel;
  open: boolean;
  onClose: () => void;
};

const CmsItemDetailsDialog: React.FC<Props> = ({
  cmsItem,
  cmsModel,
  open,
  onClose,
}) => {
  const t = useT();

  const renderFieldValue = (value: any, fieldType: string) => {
    if (value === null || value === undefined) {
      return <span className="text-muted-foreground">-</span>;
    }

    if (typeof value === "object") {
      return (
        <pre className="max-h-40 overflow-auto rounded border bg-muted p-2 text-sm">
          {JSON.stringify(value, null, 2)}
        </pre>
      );
    }

    if (fieldType === "url" && typeof value === "string") {
      return (
        <a
          href={value}
          target="_blank"
          rel="noopener noreferrer"
          className="text-blue-600 underline hover:text-blue-800">
          {value}
        </a>
      );
    }

    return <span className="break-words">{String(value)}</span>;
  };

  return (
    <Dialog open={open} onOpenChange={onClose}>
      <DialogContent className="max-h-[90vh] w-full max-w-4xl overflow-hidden">
        <DialogTitle className="flex items-center justify-between">
          <div className="flex items-center">
            <span className="font-semibold">{t("Item Details")}</span>
          </div>
        </DialogTitle>
        <DialogContentWrapper>
          <DialogContentSection className="h-[600px] overflow-hidden">
            <ScrollArea className="h-full">
              <div className="space-y-6">
                <div className="space-y-4">
                  <h3 className="text-lg font-medium">{t("Information")}</h3>
                  <div className="grid grid-cols-1 gap-4 md:grid-cols-2">
                    <div>
                      <label className="text-sm font-medium text-muted-foreground">
                        {t("ID")}
                      </label>
                      <div className="mt-1 font-mono text-sm">{cmsItem.id}</div>
                    </div>
                    <div>
                      <label className="text-sm font-medium text-muted-foreground">
                        {t("Created At")}
                      </label>
                      <div className="mt-1">{cmsItem.createdAt}</div>
                    </div>
                    <div>
                      <label className="text-sm font-medium text-muted-foreground">
                        {t("Updated At")}
                      </label>
                      <div className="mt-1">{cmsItem.updatedAt}</div>
                    </div>
                  </div>
                </div>
                <div className="space-y-4">
                  <h3 className="text-lg font-medium">{t("Fields")}</h3>
                  <div className="space-y-4">
                    {cmsModel.schema.fields.map((field) => {
                      const value = cmsItem.fields[field.key];
                      return (
                        <div
                          key={field.fieldId}
                          className="space-y-2 rounded border p-4">
                          <div className="flex items-center justify-between">
                            <div>
                              <label className="font-medium">
                                {field.name}
                              </label>
                              <div className="text-sm text-muted-foreground">
                                {field.key} • {field.type}
                              </div>
                            </div>
                          </div>
                          {field.description && (
                            <div className="text-sm text-muted-foreground">
                              {field.description}
                            </div>
                          )}
                          <div className="mt-2">
                            {renderFieldValue(value, field.type)}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
                <div className="space-y-4">
                  <h3 className="text-lg font-medium">{t("Raw Data")}</h3>
                  <pre className="overflow-auto rounded border bg-muted p-4 text-sm">
                    {JSON.stringify(cmsItem, null, 2)}
                  </pre>
                </div>
              </div>
            </ScrollArea>
          </DialogContentSection>
        </DialogContentWrapper>
      </DialogContent>
    </Dialog>
  );
};

export default CmsItemDetailsDialog;
