import { Logs } from "@flow/components/Logs";
import { logData } from "@flow/mock_data/logsData";

const LogConsole: React.FC = () => {
  const props = {
    columns: [
      {
        accessorKey: "timestamp",
        header: "Timestamp",
      },
      {
        accessorKey: "status",
        header: "Status",
      },
      {
        accessorKey: "transformer",
        header: "Transformer",
      },
      {
        accessorKey: "message",
        header: "message",
      },
    ],
    data: logData,
    selectColumns: true,
    showFiltering: true,
  };
  return <Logs {...props} />;
};

export { LogConsole };
