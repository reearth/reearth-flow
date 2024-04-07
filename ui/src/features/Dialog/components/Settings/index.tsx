import { CommitIcon, GearIcon, GroupIcon, KeyboardIcon, PersonIcon } from "@radix-ui/react-icons";

import { IconButton } from "@flow/components";
// import { Button } from "@flow/components/buttons/BaseButton";
import { DialogType } from "@flow/stores";

import {
  AccountDialogContent,
  WorkflowDialogContent,
  WorkspacesDialogContent,
  GeneralDialogContent,
  KeyboardDialogContent,
} from "./components";

type Props = {
  tab: DialogType;
  onTabChange?: (tab: DialogType) => void;
};

const SettingsDialogContent: React.FC<Props> = ({ tab, onTabChange }) => {
  return (
    <div className="flex">
      <div className="flex flex-col gap-6 pr-4 pt-10 border-r border-zinc-800">
        <IconButton
          className={`${tab === "account-settings" ? "bg-zinc-800" : undefined}`}
          tooltipText="Account settings"
          tooltipPosition="left"
          tooltipOffset={20}
          icon={<PersonIcon />}
          onClick={() => onTabChange?.("account-settings")}
        />
        <IconButton
          className={`${tab === "workspaces-settings" ? "bg-zinc-800" : undefined}`}
          tooltipText="Workspaces settings"
          tooltipPosition="left"
          tooltipOffset={20}
          icon={<GroupIcon />}
          onClick={() => onTabChange?.("workspaces-settings")}
        />
        <IconButton
          className={`${tab === "workflow-settings" ? "bg-zinc-800" : undefined}`}
          tooltipText="Workflow settings"
          tooltipPosition="left"
          tooltipOffset={20}
          icon={<CommitIcon />}
          onClick={() => onTabChange?.("workflow-settings")}
        />
        <IconButton
          className={`${tab === "keyboard-settings" ? "bg-zinc-800" : undefined}`}
          tooltipText="Keyboard shortcuts"
          tooltipPosition="left"
          tooltipOffset={20}
          icon={<KeyboardIcon />}
          onClick={() => onTabChange?.("keyboard-settings")}
        />
        <IconButton
          className={`${tab === "general-settings" ? "bg-zinc-800" : undefined}`}
          tooltipText="General settings"
          tooltipPosition="left"
          tooltipOffset={20}
          icon={<GearIcon />}
          onClick={() => onTabChange?.("general-settings")}
        />
        {/* <TabButton name="Account" />
        <TabButton name="Workspaces" />
        <TabButton name="Workflow" />
        <TabButton name="General" />
        <TabButton name="asdf" /> */}
      </div>
      <div id="settings-content" className="pl-4">
        {tab === "account-settings" ? (
          <AccountDialogContent />
        ) : tab === "workflow-settings" ? (
          <WorkflowDialogContent />
        ) : tab === "workspaces-settings" ? (
          <WorkspacesDialogContent />
        ) : tab === "keyboard-settings" ? (
          <KeyboardDialogContent />
        ) : tab === "general-settings" ? (
          <GeneralDialogContent />
        ) : null}
      </div>
    </div>
  );
};

export { SettingsDialogContent };

// const TabButton: React.FC<{ name: string }> = ({ name }) => {
//   return <Button className="bg-zinc-800">{name}</Button>;
// };
