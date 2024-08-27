import type { Shortcut } from "@flow/types";

type Props = {
  shortcuts: Shortcut[];
};

const os = window.navigator.userAgent.toLowerCase();

const Shortcuts: React.FC<Props> = ({ shortcuts }) => {
  return (
    <ul className="flex flex-col gap-2 pl-2">
      {shortcuts.map((shortcut) => (
        <li key={shortcut.keyBinding?.key} className="flex justify-between">
          <p className="font-extralight">{shortcut.description}</p>
          <div className="flex gap-1">
            <Shortcut shortcut={shortcut} />
          </div>
        </li>
      ))}
    </ul>
  );
};

export { Shortcuts };

const Shortcut = ({ shortcut }: { shortcut: Shortcut }) => {
  const symbol = shortcut.keyBinding?.commandKey
    ? os.indexOf("mac os x") !== -1
      ? "⌘"
      : "Ctrl"
    : undefined;
  return (
    <>
      {symbol && <KeyStroke keystroke={symbol} />}
      <KeyStroke keystroke={shortcut.keyBinding?.key.toUpperCase()} />
    </>
  );
};

const KeyStroke = ({ keystroke }: { keystroke?: string }) => (
  <div className="flex size-8 items-center justify-center rounded bg-zinc-700">
    <p className="text-sm font-extralight">{keystroke}</p>
  </div>
);
