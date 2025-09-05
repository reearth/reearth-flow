type Props = {
  userName?: string;
};

const CollaborationCard: React.FC<Props> = ({ userName }) => {
  return (
    <div className="flex items-center gap-2 pt-0">
      <div className="flex h-10 w-10 items-center justify-center rounded-full bg-secondary ring-background">
        <span className="text-sm font-medium">
          {userName?.charAt(0).toUpperCase()}
        </span>
      </div>

      <div className="flex flex-col items-start">
        <span className="text-sm dark:font-light">{userName}</span>
      </div>
    </div>
  );
};

export default CollaborationCard;
