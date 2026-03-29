export const paramsAwarenessStyles = (
  focusedUsers: { color: string }[] | null | undefined,
) => {
  const hasFocusedUsers =
    Array.isArray(focusedUsers) && focusedUsers.length > 0;
  return hasFocusedUsers
    ? {
        border: "2px solid",
        borderColor: focusedUsers[0]?.color,
        borderRadius: "4px",
      }
    : undefined;
};
