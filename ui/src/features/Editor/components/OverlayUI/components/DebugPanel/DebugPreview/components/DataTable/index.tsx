import { memo } from "react";

import { RenderFallback, DataTable as Table } from "@flow/components";
import useDataColumnizer from "@flow/hooks/useDataColumnizer";
import { useT } from "@flow/lib/i18n";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

type Props = {
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  selectedFeature: any;
  onSelectedFeature: (value: any) => void;
  onDoubleClickFeature?: (value: any) => void;
};

const DataTable: React.FC<Props> = ({
  fileContent,
  fileType,
  selectedFeature,
  onSelectedFeature,
  onDoubleClickFeature,
}) => {
  const { tableData, tableColumns } = useDataColumnizer({
    parsedData: fileContent,
    type: fileType,
  });
  const t = useT();
  return (
    <RenderFallback
      message={t(
        "Table Viewer Could Not Be Loaded. Check if the data is valid.",
      )}
      textSize="sm">
      <div className="flex h-full flex-1">
        <div className="mx-1 mt-0 mb-1 box-border overflow-scroll p-1">
          <Table
            columns={tableColumns}
            data={tableData}
            condensed
            selectColumns
            showFiltering
            showOrdering={false}
            selectedRow={selectedFeature}
            onRowClick={onSelectedFeature}
            onRowDoubleClick={onDoubleClickFeature}
            useStrictSelectedRow
          />
        </div>
      </div>
    </RenderFallback>
  );
};

export default memo(DataTable);
