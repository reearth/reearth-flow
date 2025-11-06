import { CopyrightIcon } from "@phosphor-icons/react";

import { attributions } from "@flow/attributions";
import {
  Dialog,
  DialogContent,
  DialogContentSection,
  DialogContentWrapper,
  DialogTitle,
} from "@flow/components";
import { useT } from "@flow/lib/i18n";

type Props = {
  isOpen: boolean;
  onOpenChange: (open: boolean) => void;
};

const AttributionsDialog: React.FC<Props> = ({ isOpen, onOpenChange }) => {
  const t = useT();


  return (
    <Dialog open={isOpen} onOpenChange={(o) => onOpenChange(o)}>
      <DialogContent className="h-[500px] w-full">
        <DialogTitle className="flex items-center gap-2">
          <CopyrightIcon /> {t("Attributions")}
        </DialogTitle>
        <DialogContentWrapper className="h-full">
          <DialogContentSection className="flex h-full flex-col gap-4 overflow-y-scroll">
            {attributions.map((attr) => (
              <div key={attr.name}>
                <h3 className="font-semibold">{attr.name}</h3>
                <p className="italic">{attr.description}</p>
                <a
                  href={attr.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-muted-foreground hover:underline">
                  {attr.url}
                </a>
              </div>
            ))}
            {attributions.map((attr) => (
              <div key={attr.name} className="mb-4">
                <h3 className="font-semibold">{attr.name}</h3>
                <p className="italic">{attr.description}</p>
                <a
                  href={attr.url}
                  target="_blank"
                  rel="noopener noreferrer"
                  className="text-muted-foreground hover:underline">
                  {attr.url}
                </a>
              </div>
            ))}
          </DialogContentSection>
        </DialogContentWrapper>
      </DialogContent>
    </Dialog>
  );
};

export { AttributionsDialog };
