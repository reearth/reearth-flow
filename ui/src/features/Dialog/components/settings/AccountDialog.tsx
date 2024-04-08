import { DialogDescription, DialogHeader, DialogTitle } from "@flow/components";

const AccountDialogContent: React.FC = () => {
  return (
    <>
      <DialogHeader>
        <DialogTitle>Account settings</DialogTitle>
        <DialogDescription className="text-wrap">
          This action cannot be undone. This will permanently delete your account and remove your
          data from our servers.
        </DialogDescription>
      </DialogHeader>
      <div className="mx-2">
        <div className="mb-4">
          <h2 className="text-lg font-semibold">Editor</h2>
          <ul className="flex flex-col gap-2 border-t border-zinc-700 px-4 py-2 mt-2">
            <li className="text-nowrap">
              <strong>⌘N</strong> - Create a new document
            </li>
            <li className="text-nowrap">
              <strong>⌘S</strong> - Save the current document
            </li>
            <li className="text-nowrap">
              <strong>⌘Z</strong> - Undo the last action
            </li>
            <li className="text-nowrap">
              <strong>⌘⇧Z</strong> - Redo the last action
            </li>
          </ul>
        </div>
        <div>
          <h2 className="text-lg font-semibold">Canvas</h2>
          <ul className="flex flex-col gap-2 border-t border-zinc-700 px-4 py-2 mt-2">
            <li className="text-nowrap">
              <strong>⌘N</strong> - Create a new document
            </li>
            <li className="text-nowrap">
              <strong>⌘S</strong> - Save the current document
            </li>
            <li className="text-nowrap">
              <strong>⌘Z</strong> - Undo the last action
            </li>
            <li className="text-nowrap">
              <strong>⌘⇧Z</strong> - Redo the last action
            </li>
          </ul>
        </div>
      </div>
      {/* <DialogFooter>
        <Button type="submit">Save changes</Button>
      </DialogFooter> */}
    </>
  );
};

export { AccountDialogContent };
