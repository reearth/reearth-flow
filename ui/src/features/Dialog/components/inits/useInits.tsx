import { StarIcon } from "@radix-ui/react-icons";

import { useT } from "@flow/providers";

import { DialogContentType } from "../Content";

import { WelcomeDialogContent } from "./";

export default (): DialogContentType[] => {
  const t = useT();
  return [
    {
      id: "welcome-init",
      title: t("Welcome to Re:Earth Flow!"),
      icon: <StarIcon />,
      component: <WelcomeDialogContent />,
    },
  ];
};
