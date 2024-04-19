import { KeyboardIcon } from "@radix-ui/react-icons";

import { useT } from "@flow/providers";

import { DialogContentType } from "../Content";

import { KeyboardDialogContent } from "./";

export default (): DialogContentType[] => {
  const t = useT();
  return [
    {
      id: "keyboard-instructions",
      title: t("Keyboard shortcuts"),
      icon: <KeyboardIcon />,
      component: <KeyboardDialogContent />,
    },
  ];
};
