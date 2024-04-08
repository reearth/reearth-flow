import { ContentHeader } from "../../ContentHeader";
import { ContentSection } from "../../ContentSection";

import { Shortcuts } from "./components";
import useShortcuts from "./useShortcuts";

const KeyboardDialogContent: React.FC = () => {
  const { title, description, editorShortcuts, canvasShortcuts } = useShortcuts();

  return (
    <>
      <ContentHeader title={title} description={description} />
      <div className="mx-2">
        <ContentSection
          title={editorShortcuts.title}
          content={<Shortcuts shortcuts={editorShortcuts.shortcuts} />}
        />
        <ContentSection
          title={canvasShortcuts.title}
          content={<Shortcuts shortcuts={canvasShortcuts.shortcuts} />}
        />
      </div>
      {/* <DialogFooter>
        <Button type="submit">Save changes</Button>
      </DialogFooter> */}
    </>
  );
};

export { KeyboardDialogContent };
