const DataTable: React.FC = () => {
  return (
    <div className="flex-1 p-2 text-zinc-400">
      <p>Data Table</p>
      <table className="table-fixed border w-[100%] border-zinc-600">
        <thead>
          <tr>
            <th className="border border-zinc-600">ID</th>
            <th className="border border-zinc-600">LAT</th>
            <th className="border border-zinc-600">LNG</th>
            <th className="border border-zinc-600">CITY</th>
          </tr>
        </thead>
        <tbody>
          <tr>
            <td className="border border-zinc-600">1234123123123</td>
            <td className="border border-zinc-600">22.12312</td>
            <td className="border border-zinc-600">80.1022922</td>
            <td className="border border-zinc-600">東京市</td>
          </tr>
          <tr>
            <td className="border border-zinc-600">lkj1234lk34jlk</td>
            <td className="border border-zinc-600">52.1312</td>
            <td className="border border-zinc-600">10.1123123123022922</td>
            <td className="border border-zinc-600">大阪市</td>
          </tr>
          <tr>
            <td className="border border-zinc-600">abcdefghijklmnopqrstuvwxyz</td>
            <td className="border border-zinc-600">72.1123121322312</td>
            <td className="border border-zinc-600">90.1022922</td>
            <td className="border border-zinc-600">横浜市</td>
          </tr>
        </tbody>
      </table>
    </div>
  );
};

export { DataTable };
