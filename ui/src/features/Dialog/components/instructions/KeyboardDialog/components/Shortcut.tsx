type Shortcut = {
  key: string;
  description: string;
};

type Props = {
  shortcuts: Shortcut[];
};

const Shortcuts: React.FC<Props> = ({ shortcuts }) => (
  <ul className="flex flex-col gap-2">
    {shortcuts.map((shortcut) => (
      <li key={shortcut.key} className="text-nowrap">
        <strong>{shortcut.key}</strong> - {shortcut.description}
      </li>
    ))}
  </ul>
);

export { Shortcuts };
