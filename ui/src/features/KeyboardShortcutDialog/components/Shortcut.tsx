import type { KeyBinding, Shortcut } from "@flow/types";

type Props = {
  shortcuts: Shortcut[];
};

const os = window.navigator.userAgent.toLowerCase();

const Shortcuts: React.FC<Props> = ({ shortcuts }) => {
  console.log(shortcuts);
  return (
    <ul className="flex flex-col gap-2 pl-2">
      {shortcuts.map(({ keyBinding, description }) => (
        <li
          key={`${keyBinding?.key}${keyBinding?.commandKey}${keyBinding?.shiftKey}${keyBinding?.altKey}`}
          className="flex justify-between">
          <p className="font-extralight">{description}</p>
          <div className="flex gap-1">
            <Shortcut keyBinding={keyBinding} />
          </div>
        </li>
      ))}
    </ul>
  );
};

export { Shortcuts };

const Shortcut = ({ keyBinding }: { keyBinding?: KeyBinding }) => {
  const commandKey = keyBinding?.commandKey
    ? os.indexOf("mac os x") !== -1
      ? "⌘"
      : "CTRL"
    : undefined;

  const shiftKey = keyBinding?.shiftKey ? "SHIFT" : undefined;
  const altKey = keyBinding?.altKey ? "ALT" : undefined;

  return (
    <>
      {commandKey && <KeyStroke keystroke={commandKey} />}
      {shiftKey && <KeyStroke keystroke={shiftKey} />}
      {altKey && <KeyStroke keystroke={altKey} />}
      <KeyStroke keystroke={keyBinding?.key.toUpperCase()} />
    </>
  );
};

const KeyStroke = ({ keystroke }: { keystroke?: string }) => (
  <div className="flex min-h-8 min-w-8 items-center justify-center rounded bg-accent px-2">
    <p className="text-sm font-extralight">{keystroke}</p>
  </div>
);
