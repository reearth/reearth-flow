import { Gear, Graph, User, UsersThree } from "@phosphor-icons/react";

import { useT } from "@flow/providers";

import { DialogContentType } from "../Content";

import {
  AccountDialogContent,
  GeneralDialogContent,
  WorkflowDialogContent,
  WorkspacesDialogContent,
} from "./";

export default (): DialogContentType[] => {
  const t = useT();
  return [
    {
      id: "account-settings",
      title: t("Account settings"),
      icon: <User />,
      component: <AccountDialogContent />,
    },
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
