import { useCallback } from "react";

import { Button, ScrollArea } from "@flow/components";
import { useCms } from "@flow/lib/gql/cms";
import { useT } from "@flow/lib/i18n";
import type {
  CmsItem,
  CmsModel,
  CmsSchemaField,
} from "@flow/types/cmsIntegration";

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
  const renderFieldValue = useCallback(
    (value: any, field?: CmsSchemaField, onSelect?: (url: string) => void) => {
      if (value === null || value === undefined) {
        return <span className="text-muted-foreground">-</span>;
      }

      if (field?.type === "asset") {
        return <AssetValue value={value} onSelect={onSelect} />;
      }

      if (typeof value === "object") {
        return (
          <pre className="max-h-40 overflow-auto rounded border bg-muted p-2 text-sm">
            {JSON.stringify(value, null, 2)}
          </pre>
        );
      }

      return <span className="wrap-break-word">{String(value)}</span>;
    },
    [],
  );

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

                if (field.type === "asset") {
                  return (
                    <AssetDetail
                      key={field.fieldId}
                      field={field}
                      value={value}
                      renderFieldValue={renderFieldValue}
                      onCmsItemValue={onCmsItemValue}
                    />
                  );
                } else {
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
                        <div>{renderFieldValue(value, field)}</div>
                      </div>
                      {field.type === "url" && value && (
                        <Button
                          className="self-center"
                          onClick={() => onCmsItemValue?.(value)}>
                          {t("Select")}
                        </Button>
                      )}
                    </div>
                  );
                }
              })}
            </div>
          </div>
        </div>
      </ScrollArea>
    </div>
  );
};
const AssetDetail: React.FC<{
  field: CmsSchemaField;
  value: any;
  renderFieldValue: (
    value: any,
    field?: CmsSchemaField,
    onSelect?: (url: string) => void,
  ) => React.ReactNode;
  onCmsItemValue?: (value: string) => void;
}> = ({ field, value, renderFieldValue, onCmsItemValue }) => {
  return (
    <div className="rounded border p-4">
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
        <div>{renderFieldValue(value, field, onCmsItemValue)}</div>
      </div>
    </div>
  );
};

const AssetValue: React.FC<{
  value: any;
  onSelect?: (url: string) => void;
}> = ({ value, onSelect }) => {
  // Handle array of asset IDs
  if (
    typeof value === "string" &&
    value.startsWith("[") &&
    value.endsWith("]")
  ) {
    const cleanedArray = value
      .split(",")
      .map((v: string) => v?.replace(/[[\]\s"]/g, ""))
      .filter(Boolean);

    return (
      <div className="flex flex-col gap-2">
        {cleanedArray.map((assetId: string) => (
          <AssetFilename key={assetId} assetId={assetId} onSelect={onSelect} />
        ))}
      </div>
    );
  }

  if (value) {
    return <AssetFilename assetId={value} onSelect={onSelect} />;
  }

  return <span className="text-muted-foreground">-</span>;
};

const AssetFilename: React.FC<{
  assetId: string;
  onSelect?: (url: string) => void;
}> = ({ assetId, onSelect }) => {
  const t = useT();
  const { useGetCmsAsset } = useCms();
  const cleanedAssetId = assetId.replace(/[^a-zA-Z0-9]/g, "");
  const { cmsAsset, isLoading } = useGetCmsAsset(cleanedAssetId);

  const handleClick = () => {
    if (cmsAsset?.url) {
      onSelect?.(cmsAsset.url);
    }
  };

  if (isLoading) {
    return (
      <span className="text-sm text-muted-foreground">{t("Loading...")}</span>
    );
  }

  if (!cmsAsset) {
    return <span className="text-sm wrap-break-word">{assetId}</span>;
  }

  return (
    <div className="flex items-center">
      {isLoading ? (
        <span className="text-sm text-muted-foreground">{t("Loading...")}</span>
      ) : (
        <div className="flex flex-1 flex-col gap-1">
          <span className="font-medium wrap-break-word">
            {cmsAsset.filename}
          </span>
          <span className="font-mono text-xs wrap-break-word text-muted-foreground">
            {assetId}
          </span>
        </div>
      )}

      {onSelect && (
        <Button onClick={handleClick} disabled={isLoading || !cmsAsset?.url}>
          {t("Select")}
        </Button>
      )}
    </div>
  );
};

export default CmsItemDetails;
