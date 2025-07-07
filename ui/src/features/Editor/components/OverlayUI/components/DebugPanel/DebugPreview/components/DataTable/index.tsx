import { RenderFallback, DataTable as Table } from "@flow/components";
import useDataColumnizer from "@flow/hooks/useDataColumnizer";
import { useT } from "@flow/lib/i18n";
import { SupportedDataTypes } from "@flow/utils/fetchAndReadGeoData";

type Props = {
  fileContent: any | null;
  fileType: SupportedDataTypes | null;
};

const DataTable: React.FC<Props> = ({ fileContent, fileType }) => {
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
      <div className="box-border flex h-full flex-1">
        <div className="mx-1 mt-0 mb-1 box-border flex-1 overflow-scroll">
          <Table
            columns={tableColumns}
            data={tableData}
            condensed
            selectColumns
            showFiltering
          />
        </div>
      </div>
    </RenderFallback>
  );
};

export { DataTable };
