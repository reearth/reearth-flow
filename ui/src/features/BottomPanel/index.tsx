import { HorizontalPanel, OutputIcon, PreviewIcon, type PanelContent } from "@flow/components";
import { useStateManager } from "@flow/hooks";

export type BottomPanelProps = {
  className?: string;
};

const BottomPanel: React.FC<BottomPanelProps> = ({ className }) => {
  const [isPanelOpen, handlePanelToggle] = useStateManager(false);

  const panelContents: PanelContent[] = [
    {
      id: "translation-log",
      icon: <OutputIcon />,
      component: (
        <div className="bg-zinc-900 text-yellow-600 h-[204px] w-[100%] overflow-scroll rounded-md p-1">
          <ol>
            <li>.....asldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa123.....</li>
            <li>
              dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>
              .....asldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa.....dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd</li>
            <li>.....asldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa123.....</li>
            <li>
              dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>
              .....asldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa.....dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd</li>
            <li>.....asldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa123.....</li>
            <li>
              dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfddadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>
              .....asldfkjasldfkjsf....aslkdfjalskdfjasldfkjsdfa.....dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd
            </li>
            <li>dadfasdfsad......asdf..asdf.asdfasdf.......asdfsfd</li>
          </ol>
        </div>
      ),
    },
    {
      id: "visual-preview",
      icon: <PreviewIcon />,
      component: (
        <div>
          <p>This is preview</p>
        </div>
      ),
    },
  ];

  return (
    <HorizontalPanel
      className={`bg-zinc-950 mb-1 rounded-md ${className} ${!isPanelOpen ? "cursor-pointer" : undefined}`}
      isOpen={!!isPanelOpen}
      panelContents={panelContents}
      onClick={currentState => !isPanelOpen && handlePanelToggle(!currentState)}
      onPanelToggle={handlePanelToggle}
    />
  );
};

export default BottomPanel;
