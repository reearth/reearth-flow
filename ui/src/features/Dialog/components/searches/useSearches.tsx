import { StarIcon } from "@radix-ui/react-icons";

import { useT } from "@flow/lib/i18n";

import { DialogContentType } from "../Content";

import { CanvasSearch } from "./CanvasSearch";

export default (): DialogContentType[] => {
  const t = useT();
  return [
    {
      id: "canvas-search",
      title: t("Canvas search"),
      icon: <StarIcon />,
      component: <CanvasSearch />,
    },
  ];
};
