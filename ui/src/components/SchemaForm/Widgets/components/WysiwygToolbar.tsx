import {
  ArrowArcLeftIcon,
  ArrowArcRightIcon,
  CodeIcon,
  LinkSimpleHorizontalIcon,
  TextBIcon,
  TextItalicIcon,
  TextStrikethroughIcon,
  TextUnderlineIcon,
} from "@phosphor-icons/react";
import { createButton, Toolbar } from "react-simple-wysiwyg";

import { useT } from "@flow/lib/i18n";

export const WysiwygToolbar = () => {
  const t = useT();

  const BtnBold = createButton(t("Bold"), <TextBIcon />, "bold");

  const BtnUndo = createButton(t("Undo"), <ArrowArcLeftIcon />, "undo");
  const BtnRedo = createButton(t("Redo"), <ArrowArcRightIcon />, "redo");

  const BtnUnderline = createButton(
    t("Underline"),
    <TextUnderlineIcon />,
    "underline",
  );

  const BtnItalic = createButton(t("Italic"), <TextItalicIcon />, "italic");

  const BtnLink = createButton(
    t("Link"),
    <LinkSimpleHorizontalIcon />,
    ({ $selection }) => {
      if ($selection?.nodeName === "A") {
        document.execCommand("unlink");
      } else {
        document.execCommand(
          "createLink",
          false,
          prompt(t("URL"), "") || undefined,
        );
      }
    },
  );

  const BtnStrikeThrough = createButton(
    t("Strike through"),
    <TextStrikethroughIcon />,
    "strikeThrough",
  );

  const BtnCode = createButton(t("Code"), <CodeIcon />, "PRE");

  // const BtnNumberedList = createButton(
  //   t("Numbered list"),
  //   <ListNumbersIcon />,
  //   "insertOrderedList",
  // );

  return (
    <Toolbar>
      <BtnUndo />
      <BtnRedo />
      <BtnBold />
      <BtnUnderline />
      <BtnItalic />
      <BtnLink />
      <BtnStrikeThrough />
      {/* <BtnStyles /> */}
      <BtnCode />
    </Toolbar>
  );
};
