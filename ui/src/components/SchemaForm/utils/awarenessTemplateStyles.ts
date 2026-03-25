export const paramsAwarenessStyles = (
  focusedUsers: { color: string }[] | null | undefined,
) => ({
  border:
    Array.isArray(focusedUsers) && focusedUsers.length > 0
      ? "2px solid"
      : undefined,
  borderColor:
    Array.isArray(focusedUsers) && focusedUsers.length > 0
      ? focusedUsers[0]?.color
      : undefined,
  borderRadius:
    Array.isArray(focusedUsers) && focusedUsers.length > 0 ? "4px" : undefined,
});
