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
  onClose: () => void;
};

const PaneContextMenu: React.FC<Props> = ({
  contextMenu,
  onCopy,
  onPaste,
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

          onCallback: wrapWithClose(() => onPaste?.(contextMenu.mousePosition)),
        },
      },
    ];

    return items;
  }, [t, onCopy, onPaste, onClose, contextMenu.mousePosition]);

  return <ContextMenu items={menuItems} contextMenuMeta={contextMenu} />;
};

export default PaneContextMenu;
