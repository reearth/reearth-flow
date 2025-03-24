import { DataTable as Table } from "@flow/components";
import useDataColumnizer from "@flow/hooks/useDataColumnizer";
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
  return (
    <div className="box-border flex h-full flex-1">
      <div className="mx-1 mb-1 mt-0 box-border flex-1 overflow-scroll">
        <Table
          columns={tableColumns}
          data={tableData}
          condensed
          selectColumns
          showFiltering
        />
      </div>
    </div>
  );
};

export { DataTable };
