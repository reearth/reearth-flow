import { Gear, Graph, UsersThree } from "@phosphor-icons/react";

import { useT } from "@flow/lib/i18n";

import { DialogContentType } from "../Content";

import {
  GeneralDialogContent,
  WorkflowDialogContent,
  WorkspacesDialogContent,
} from "./";

export default (): DialogContentType[] => {
  const t = useT();
  return [
    {
      id: "workspaces-settings",
      title: t("Workspaces settings"),
      icon: <UsersThree />,
      component: <WorkspacesDialogContent />,
    },
    {
      id: "project-settings",
      title: t("Project settings"),
      icon: <Graph />,
      component: <WorkflowDialogContent />,
    },
    {
      id: "general-settings",
      title: t("General settings"),
      icon: <Gear />,
      component: <GeneralDialogContent />,
    },
  ];
};
