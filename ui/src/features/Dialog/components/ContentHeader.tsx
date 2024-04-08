import { DialogDescription, DialogHeader, DialogTitle } from "@flow/components";

type Props = {
  title: string;
  description?: string;
};

const ContentHeader: React.FC<Props> = ({ title, description }) => (
  <DialogHeader className="mb-4">
    <DialogTitle>{title}</DialogTitle>
    {description && (
      <DialogDescription className="text-wrap text-center">{description}</DialogDescription>
    )}
  </DialogHeader>
);

export { ContentHeader };
