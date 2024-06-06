import { CaretRight } from "@phosphor-icons/react";

import { useT } from "@flow/lib/i18n";

import { DialogContentType } from "../Content";

import { AddProject } from "./AddProject";

export default (): DialogContentType[] => {
  const t = useT();
  return [
    {
      id: "add-project",
      title: t("Add Project"),
      icon: <CaretRight />,
      component: <AddProject />,
    },
  ];
};
