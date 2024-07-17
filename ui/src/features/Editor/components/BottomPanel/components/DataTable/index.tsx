import { ColumnDef } from "@tanstack/react-table";

import { DataTable as Table } from "@flow/components";
import { points } from "@flow/mock_data/pointData";

// TODO: This is just placeholder code at the moment
// In production this will either be infered dynamically or a fixed type based on the API implementation.
type Fire = {
  ACQ_DATE: string;
  ACQ_TIME: string;
  BRIGHT_TI4: number;
  BRIGHT_TI5: number;
  CONFIDENCE: string;
  DAYNIGHT: string;
  FRP: number;
  LATITUDE: number;
  LONGITUDE: number;
  SATELLITE: string;
  SCAN: number;
  TRACK: number;
  VERSION: string;
};

const columns: ColumnDef<Fire>[] = [
  {
    accessorKey: "ACQ_DATE",
    header: "ACQ_DATE",
  },
  {
    accessorKey: "ACQ_TIME",
    header: "ACQ_TIME",
  },
  {
    accessorKey: "BRIGHT_TI4",
    header: "BRIGHT_TI4",
  },
  {
    accessorKey: "BRIGHT_TI5",
    header: "BRIGHT_TI5",
  },
  {
    accessorKey: "CONFIDENCE",
    header: "CONFIDENCE",
  },
  {
    accessorKey: "DAYNIGHT",
    header: "DAYNIGHT",
  },
  {
    accessorKey: "FRP",
    header: "FRP",
  },
  {
    accessorKey: "LATITUDE",
    header: "LATITUDE",
  },
  {
    accessorKey: "LONGITUDE",
    header: "LONGITUDE",
  },
  {
    accessorKey: "SATELLITE",
    header: "SATELLITE",
  },
  {
    accessorKey: "SCAN",
    header: "SCAN",
  },
  {
    accessorKey: "TRACK",
    header: "TRACK",
  },
  {
    accessorKey: "VERSION",
    header: "VERSION",
  },
];
const data: Fire[] = points;

const DataTable: React.FC = () => {
  return (
    <div className="container mx-auto w-6/12 overflow-auto">
      <Table columns={columns} data={data} selectColumns showFiltering />
    </div>
  );
};

export { DataTable };
