import {
  Circle,
  Database,
  Disc,
  Graph,
  Lightning,
  Note,
} from "@phosphor-icons/react";

export function getNodeIcon(type: string | undefined) {
  switch (type) {
    case "note":
      return Note;
    case "subworkflow":
      return Graph;
    case "transformer":
      return Lightning;
    case "reader":
      return Database;
    case "writer":
      return Disc;
    default:
      return Circle;
  }
}
