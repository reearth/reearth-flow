import { Clipboard, Copy } from "@phosphor-icons/react";
import { XYPosition } from "@xyflow/react";
import { useMemo } from "react";

import {
  ContextMenu,
  ContextMenuItemType,
  ContextMenuMeta,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  contextMenu: ContextMenuMeta;
  onCopy?: () => void;
  onPaste?: (menuPosition?: XYPosition) => void;
  hasItemsToPaste?: boolean;
  onClose: () => void;
};

const PaneContextMenu: React.FC<Props> = ({
  contextMenu,
  onCopy,
  onPaste,
  hasItemsToPaste,
  onClose,
}) => {
  const t = useT();

  const menuItems = useMemo(() => {
    const wrapWithClose = (callback: () => void) => () => {
      callback();
      onClose();
    };

    const items: ContextMenuItemType[] = [
      {
        type: "action",
        props: {
          label: t("Copy"),
          icon: <Copy weight="light" />,
          disabled: true,
          onCallback: wrapWithClose(onCopy ?? (() => {})),
        },
      },
      {
        type: "action",
        props: {
          label: t("Paste"),
          icon: <Clipboard weight="light" />,
          disabled: !hasItemsToPaste,
          onCallback: wrapWithClose(onPaste ?? (() => {})),
        },
      },
    ];

    return items;
  }, [t, onCopy, onPaste, hasItemsToPaste, onClose]);

  return <ContextMenu items={menuItems} contextMenuMeta={contextMenu} />;
};

export default PaneContextMenu;
