import fires from "./fires.json";

const points = fires.features.map(f => ({
  ...f.properties,
}));

export { points };
