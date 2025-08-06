import { memo, useState } from "react";

import { RenderFallback } from "@flow/components";
import { VirtualizedTable } from "@flow/components/visualizations/VirtualizedTable";
import useDataColumnizer from "@flow/hooks/useDataColumnizer";
import { useT } from "@flow/lib/i18n";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

type Props = {
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
  selectedFeature: any;
  onSingleClick: (value: any) => void;
  onDoubleClick?: (value: any) => void;
};

const TabularView: React.FC<Props> = ({
  fileContent,
  fileType,
  selectedFeature,
  onSingleClick,
  onDoubleClick,
}) => {
  const { tableData, tableColumns } = useDataColumnizer({
    parsedData: fileContent,
    type: fileType,
  });
  const t = useT();
  const [searchTerm, setSearchTerm] = useState<string>("");
  return (
    <RenderFallback
      message={t(
        "Table Viewer Could Not Be Loaded. Check if the data is valid.",
      )}
      textSize="sm">
      <div className="flex h-full flex-1">
        <div className="overflow-scroll">
          <VirtualizedTable
            columns={tableColumns}
            data={tableData}
            condensed
            selectColumns
            showFiltering
            searchTerm={searchTerm}
            setSearchTerm={setSearchTerm}
            selectedRow={selectedFeature}
            onRowClick={onSingleClick}
            onRowDoubleClick={onDoubleClick}
            useStrictSelectedRow
          />
        </div>
      </div>
    </RenderFallback>
  );
};

export default memo(TabularView);
