import { CommitIcon, GearIcon, GroupIcon, PersonIcon } from "@radix-ui/react-icons";

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
      icon: <PersonIcon />,
      component: <AccountDialogContent />,
    },
    {
      id: "workspaces-settings",
      title: t("Workspaces settings"),
      icon: <GroupIcon />,
      component: <WorkspacesDialogContent />,
    },
    {
      id: "workflow-settings",
      title: t("Workflow settings"),
      icon: <CommitIcon />,
      component: <WorkflowDialogContent />,
    },
    {
      id: "general-settings",
      title: t("General settings"),
      icon: <GearIcon />,
      component: <GeneralDialogContent />,
    },
  ];
};
