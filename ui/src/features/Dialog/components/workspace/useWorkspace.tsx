import { CaretRight } from "@phosphor-icons/react";

import { useT } from "@flow/lib/i18n";

import { DialogContentType } from "../Content";

import { AddWorkspace } from "./AddWorkspace";

export default (): DialogContentType[] => {
  const t = useT();
  return [
    {
      id: "add-workspace",
      title: t("Add Workspace"),
      icon: <CaretRight />,
      component: <AddWorkspace />,
    },
  ];
};
