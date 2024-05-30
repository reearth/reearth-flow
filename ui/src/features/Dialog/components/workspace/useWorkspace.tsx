import { StarIcon } from "@radix-ui/react-icons";

import { useT } from "@flow/providers";

import { DialogContentType } from "../Content";

import { AddWorkspace } from "./AddWorkspace";

export default (): DialogContentType[] => {
  const t = useT();
  return [
    {
      id: "add-workspace",
      title: t("Add Workspace"),
      icon: <StarIcon />,
      component: <AddWorkspace />,
    },
  ];
};
