import type { KeyBinding, Shortcut } from "@flow/types";

type Props = {
  shortcuts: Shortcut[];
};

const os = window.navigator.userAgent.toLowerCase();

const Shortcuts: React.FC<Props> = ({ shortcuts }) => (
  <ul className="flex flex-col">
    {shortcuts.map(({ keyBinding, description }, idx) => (
      <li
        key={`${keyBinding?.key}${keyBinding?.commandKey}${keyBinding?.shiftKey}${keyBinding?.altKey}`}
        className={`flex items-center justify-between rounded-md p-1 ${idx % 2 === 0 ? "bg-primary/50" : undefined}`}>
        <p className="dark:font-extralight">{description}</p>
        <div className="flex gap-1">
          <Shortcut keyBinding={keyBinding} />
        </div>
      </li>
    ))}
  </ul>
);

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
  <div className="flex min-h-7 min-w-7 items-center justify-center rounded bg-accent px-2">
    <p className="text-sm dark:font-extralight">{keystroke}</p>
  </div>
);
