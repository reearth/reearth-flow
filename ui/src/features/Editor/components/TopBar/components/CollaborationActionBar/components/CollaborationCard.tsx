type AwarnessUser = {
  userId: string;
  userName: string;
  color?: string;
  displayPictureUrl?: string;
  lastActive?: string;
};
type Props = {
  user?: AwarnessUser;
};

const CollaborationCard: React.FC<Props> = ({ user }) => {
  return (
    <div className="flex items-center gap-2 pt-0">
      {user?.displayPictureUrl ? (
        <img
          key={user.userId}
          className="inline-block h-10 w-10 rounded-full ring-background"
          src={user.displayPictureUrl}
          alt="User Avatar"
        />
      ) : (
        <div
          key={user?.userId}
          className="flex h-10 w-10 items-center justify-center rounded-full bg-secondary ring-background">
          <span className="text-sm font-medium">
            {user?.userName.charAt(0).toUpperCase()}
          </span>
        </div>
      )}
      <div className="flex flex-col items-start">
        <span className="text-sm dark:font-light">{user?.userName}</span>
      </div>
    </div>
  );
};

export default CollaborationCard;
