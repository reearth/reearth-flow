import {
  CircleIcon,
  DatabaseIcon,
  DiscIcon,
  GraphIcon,
  LightningIcon,
  NoteIcon,
} from "@phosphor-icons/react";

export function getNodeIcon(type: string | undefined) {
  switch (type) {
    case "note":
      return NoteIcon;
    case "subworkflow":
      return GraphIcon;
    case "transformer":
      return LightningIcon;
    case "reader":
      return DatabaseIcon;
    case "writer":
      return DiscIcon;
    default:
      return CircleIcon;
  }
}
