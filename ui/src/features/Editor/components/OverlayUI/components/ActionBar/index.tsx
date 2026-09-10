import {
  ClockCounterClockwiseIcon,
  DotsThreeVerticalIcon,
  ExportIcon,
  PaperPlaneTiltIcon,
  RocketIcon,
  FloppyDiskIcon,
  LockIcon,
  LockOpenIcon,
} from "@phosphor-icons/react";
import { memo } from "react";

import {
  ContextMenuShortcut,
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
  IconButton,
  Popover,
  PopoverContent,
  PopoverTrigger,
} from "@flow/components";
import {
  useEchoDropdown,
  useEchoHover,
  useEditorContext,
  useIsReadOnly,
} from "@flow/features/Editor/editorContext";
import { useT } from "@flow/lib/i18n";
import { actionItemEchoKey, ECHO_KEYS } from "@flow/lib/yjs";
import { awarenessEchoStyles } from "@flow/utils";

import { DialogOptions } from "../../types";

import { DeployPopover, SharePopover } from "./components";

const tooltipOffset = 6;

/**
 * Menu entry that rings in a collaborator's colour while they are on it. Its own
 * component because each entry needs its own `useEchoHover` subscription.
 */
const EchoMenuItem: React.FC<
  React.ComponentProps<typeof DropdownMenuItem> & { echoKey: string }
> = ({ echoKey, style, children, ...props }) => {
  const { users, hoverProps } = useEchoHover(echoKey);
  return (
    <DropdownMenuItem
      style={{ ...style, ...awarenessEchoStyles(users) }}
      {...hoverProps}
      {...props}>
      {children}
    </DropdownMenuItem>
  );
};

type Props = {
  allowedToDeploy: boolean;
  isSaving: boolean;
  showDialog: DialogOptions;
  onDialogOpen: (dialog: DialogOptions) => void;
  onDialogClose: () => void;
  onWorkflowDeployment: (
    description: string,
    deploymentId?: string,
  ) => Promise<void>;
  sharingUrl?: string;
  onProjectShare: (share: boolean) => void;
  onProjectExport: () => void;
  onProjectSnapshotSave: () => Promise<void>;
  onProjectLockChange: (lock: boolean) => void;
};

const ActionBar: React.FC<Props> = ({
  allowedToDeploy,
  isSaving,
  showDialog,
  onDialogOpen,
  onDialogClose,
  onWorkflowDeployment,
  sharingUrl,
  onProjectShare,
  onProjectExport,
  onProjectSnapshotSave,
  onProjectLockChange,
}) => {
  const t = useT();
  const { isLocked, isReaderRestricted } = useEditorContext();
  const readonly = useIsReadOnly();
  const actionsDropdown = useEchoDropdown(ECHO_KEYS.actionsDropdown);

  return (
    <div className="flex gap-2 align-middle">
      <Popover
        open={showDialog === "deploy"}
        onOpenChange={(open) => {
          if (!open) onDialogClose();
        }}>
        <PopoverTrigger
          render={
            <IconButton
              tooltipText={t("Deploy project's workflow")}
              tooltipOffset={tooltipOffset}
              disabled={isReaderRestricted}
              icon={<RocketIcon weight="thin" size={18} />}
              onClick={() => onDialogOpen("deploy")}
            />
          }
        />
        <PopoverContent
          sideOffset={8}
          collisionPadding={5}
          className="bg-primary/50 backdrop-blur">
          {showDialog === "deploy" && (
            <DeployPopover
              allowedToDeploy={allowedToDeploy}
              onWorkflowDeployment={onWorkflowDeployment}
              onDialogClose={onDialogClose}
            />
          )}
        </PopoverContent>
      </Popover>
      <Popover
        open={showDialog === "share"}
        onOpenChange={(open) => {
          if (!open) onDialogClose();
        }}>
        <PopoverTrigger
          render={
            <IconButton
              tooltipText={t("Share Project")}
              tooltipOffset={tooltipOffset}
              icon={<PaperPlaneTiltIcon weight="thin" size={18} />}
              onClick={() => onDialogOpen("share")}
            />
          }
        />
        <PopoverContent
          sideOffset={8}
          collisionPadding={5}
          className="bg-primary/50 backdrop-blur">
          {showDialog === "share" && (
            <SharePopover
              readonly={isReaderRestricted}
              sharingUrl={sharingUrl}
              onProjectShare={onProjectShare}
            />
          )}
        </PopoverContent>
      </Popover>
      <DropdownMenu {...actionsDropdown}>
        <DropdownMenuTrigger
          render={
            <IconButton
              className="w-[25px]"
              tooltipText={t("Additional actions")}
              tooltipOffset={tooltipOffset}
              icon={<DotsThreeVerticalIcon size={18} />}
            />
          }
        />
        <DropdownMenuContent
          className="min-w-[170px] bg-primary/50 backdrop-blur select-none"
          align="end"
          sideOffset={8}
          alignOffset={2}>
          <EchoMenuItem
            echoKey={actionItemEchoKey("manual-save")}
            className="flex items-center justify-between"
            closeOnClick={false}
            disabled={isSaving || readonly}
            onClick={onProjectSnapshotSave}>
            <div className="flex items-center gap-1">
              <FloppyDiskIcon weight="light" />
              <p>{t("Manual Save")}</p>
            </div>
            <div className="flex flex-row gap-1">
              <ContextMenuShortcut
                keyBinding={{ key: "s", commandKey: true }}
              />
            </div>
          </EchoMenuItem>
          <EchoMenuItem
            echoKey={actionItemEchoKey("lock")}
            className="flex items-center justify-between"
            closeOnClick={false}
            disabled={isReaderRestricted}
            onClick={onProjectLockChange?.bind(null, !isLocked)}>
            <div className="flex items-center gap-1">
              {isLocked ? (
                <LockIcon weight="light" />
              ) : (
                <LockOpenIcon weight="light" />
              )}
              <p>{isLocked ? t("Unlock Project") : t("Lock Project")}</p>
            </div>
            <div className="flex flex-row gap-1">
              <ContextMenuShortcut
                keyBinding={{ key: "l", commandKey: true }}
              />
            </div>
          </EchoMenuItem>
          <DropdownMenuSeparator />
          <EchoMenuItem
            echoKey={actionItemEchoKey("version-history")}
            className="flex items-center justify-between"
            onClick={() => onDialogOpen("version")}>
            <div className="flex items-center gap-1">
              <ClockCounterClockwiseIcon weight="light" />
              <p>{t("Version History")}</p>
            </div>
          </EchoMenuItem>
          <EchoMenuItem
            echoKey={actionItemEchoKey("export")}
            className="flex items-center justify-between"
            onClick={onProjectExport}
            disabled>
            <div className="flex items-center gap-1">
              <ExportIcon weight="light" />
              <p>{t("Export Project")}</p>
            </div>
          </EchoMenuItem>
        </DropdownMenuContent>
      </DropdownMenu>
    </div>
  );
};

export default memo(ActionBar);
