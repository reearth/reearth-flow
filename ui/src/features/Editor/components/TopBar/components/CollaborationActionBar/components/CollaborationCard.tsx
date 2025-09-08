type Props = {
  userName: string;
  color: string;
};

const CollaborationCard: React.FC<Props> = ({ userName, color }) => {
  return (
    <div className="flex items-center gap-2 pt-0">
      <div
        className="flex h-10 w-10 items-center justify-center rounded-full ring-2 ring-secondary/20"
        style={{ backgroundColor: color }}>
        <span className="text-sm font-medium">
          {userName.charAt(0).toUpperCase()}
        </span>
      </div>
      <div className="flex flex-col items-start">
        <span className="text-sm dark:font-light">{userName}</span>
      </div>
    </div>
  );
};

export default CollaborationCard;
