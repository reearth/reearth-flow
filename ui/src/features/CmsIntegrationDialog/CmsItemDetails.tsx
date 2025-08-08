import { useCallback } from "react";

import { Button, ScrollArea } from "@flow/components";
import { useT } from "@flow/lib/i18n";
import type { CmsItem, CmsModel } from "@flow/types/cmsIntegration";

type Props = {
  cmsItem: CmsItem;
  cmsModel: CmsModel;
  onCmsItemValue?: (value: string) => void;
};

const CmsItemDetails: React.FC<Props> = ({
  cmsItem,
  cmsModel,
  onCmsItemValue,
}) => {
  const t = useT();
  const renderFieldValue = useCallback((value: any) => {
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

    return <span className="break-words">{String(value)}</span>;
  }, []);

  return (
    <div className="flex h-[600px] flex-col gap-4 overflow-hidden">
      <div className="space-y-4">
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
      <ScrollArea className="h-full flex-1">
        <div className="space-y-6">
          <div className="space-y-4">
            <h3 className="text-lg font-medium">{t("Fields")}</h3>
            <div className="space-y-4">
              {cmsModel.schema.fields.map((field) => {
                const value = cmsItem.fields[field.key];
                return (
                  <div
                    key={field.fieldId}
                    className="flex justify-between space-y-2 rounded border p-4">
                    <div className="flex flex-col gap-2">
                      <div className="flex items-center justify-between">
                        <div>
                          <label className="font-medium">{field.name}</label>
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
                      <div>{renderFieldValue(value)}</div>
                    </div>
                    {(field.type === "asset" || field.type === "url") &&
                      value && (
                        <Button
                          className="self-center"
                          onClick={() => onCmsItemValue?.(value)}>
                          {t("Select Asset")}
                        </Button>
                      )}
                  </div>
                );
              })}
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  );
};

export default CmsItemDetails;
