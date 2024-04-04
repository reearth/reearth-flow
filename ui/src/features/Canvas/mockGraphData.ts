type Property = any; // maybe we can type this better later. Maybe

type CommonNode = {
  id: string;
  name: string;
  with?: Property;
};

type ActionNode = CommonNode & {
  id: string;
  type: "action";
  action: string; // "fileReader" || "entityFilter" || "fileWriter" || etc
};

type SubgraphNode = CommonNode & {
  type: "subgraph";
  subgraphId: string;
};

type Node = ActionNode | SubgraphNode;

type Edge = {
  id: string;
  from: string;
  to: string;
  fromPort: string;
  toPort: string; // "default" | "supplier" | "requestor" | "consumer" | etc
};

export const nodesdata: Node[] = [
  {
    id: "asldfjlka-asdflkjasdf-asdlfkjasdf-lkj1234",
    name: "Source-Sample-DatasetUrl-01",
    type: "action",
    action: "fileReader",
    with: {
      format: "csv",
      dataset: "https://editor.nodenodenode.net/sampleData/e-stat-0000010201-en.csv",
      header: true,
      offset: 28,
    },
  },
  {
    id: "asldfjlka-asdflkjasdf-asdlfkjasdf-lkj5678",
    name: "Ops-StringSpread-Filter",
    type: "action",
    action: "entityFilter",
    with: {
      conditions: {
        expression: `env.get("SURVEY YEAR") == "2019" && env.get("AREA") != "All Japan" && env.get("A Population and Households") == "#A011000_Total population"`,
        outputPort: "default",
      },
    },
  },
  {
    id: "asldfjlka-asdflkjasdf-asdlfkjasdf-lkj910234",
    name: "Source-DatasetOps-AppendJapanCitiesCoords-02",
    type: "action",
    action: "attributeManager",
    with: {
      operations: {
        attribute: "admin_name",
        method: "convert",
        value: `switch env.get("admin_name") {
            "Tokyo" => "Tokyo-to", "Osaka" | "Kyoto" => env.get("admin_name") + "-fu",
            "Hokkaido" => "Hokkaido", "Gunma" => "Gumma-ken",
            _ => env.get("admin_name") + "-ken"
          }`,
      },
    },
  },
  {
    id: "asldfjlka-asdflkjasdf-asdlfkjasdf-lkj",
    name: "Entity-CZML-Renderer",
    type: "action",
    action: "fileWriter",
    with: {
      format: "json",
      output: "file://laskfjsdlfka/output.json",
    },
  },
];

export const edgesdata: Edge[] = [
  {
    id: "alsdfj-asdfjlk-asdlfkj-1234",
    from: "asldfjlka-asdflkjasdf-asdlfkjasdf-lkj1234",
    to: "asldfjlka-asdflkjasdf-asdlfkjasdf-lkj5678",
    fromPort: "default",
    toPort: "default",
  },
  {
    id: "alsdfj-asdfjlk-asdlfkj-5678",
    from: "asldfjlka-asdflkjasdf-asdlfkjasdf-lkj5678",
    to: "asldfjlka-asdflkjasdf-asdlfkjasdf-lkj910234",
    fromPort: "default",
    toPort: "supplier",
  },
  {
    id: "alsdfj-asdfjlk-asdlfkj-910234",
    from: "asldfjlka-asdflkjasdf-asdlfkjasdf-lkj910234",
    to: "asldfjlka-asdflkjasdf-asdlfkjasdf-lkj",
    fromPort: "default",
    toPort: "default",
  },
];

export const graphdata = {
  id: "test-graph",
  name: "Test Graph",
  nodes: nodesdata,
  edges: edgesdata,
};
