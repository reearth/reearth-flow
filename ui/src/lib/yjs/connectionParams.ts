// The websocket server checks one credential per connection: the user's
// access token on project pages, or the project's share token on a shared page
// (which connects read-only).
export async function connectionParams({
  isProtected,
  shareToken,
  getAccessToken,
}: {
  isProtected?: boolean;
  shareToken?: string;
  getAccessToken: () => Promise<string>;
}): Promise<Record<string, string>> {
  if (isProtected) return { token: await getAccessToken() };
  if (shareToken) return { shareToken };
  return {};
}
