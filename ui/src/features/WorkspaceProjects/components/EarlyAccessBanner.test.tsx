import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, test, vi } from "vitest";

import { EarlyAccessBanner } from "./EarlyAccessBanner";

const me = { id: "u1" };

vi.mock("@flow/lib/gql", () => ({
  useUser: () => ({ useGetMe: () => ({ me }) }),
}));

vi.mock("@flow/config", () => ({
  config: () => ({
    feedbackUrl: "https://example.com/feedback",
    earlyAccessUrl: "https://example.com/early-access",
  }),
}));

const message = "Flow is in early access. Your feedback helps us improve it.";

describe("EarlyAccessBanner", () => {
  beforeEach(() => window.localStorage.clear());

  test("links to the feedback page in a new tab", () => {
    render(<EarlyAccessBanner />);

    const link = screen.getByRole("link", { name: "Share feedback" });
    expect(link).toHaveAttribute("href", "https://example.com/feedback");
    expect(link).toHaveAttribute("target", "_blank");
  });

  test("links to the early access notes in a new tab", () => {
    render(<EarlyAccessBanner />);

    const link = screen.getByRole("link", {
      name: "Learn more about early access",
    });
    expect(link).toHaveAttribute("href", "https://example.com/early-access");
    expect(link).toHaveAttribute("target", "_blank");
  });

  test("stays closed for the user after Close", () => {
    const { unmount } = render(<EarlyAccessBanner />);
    fireEvent.click(screen.getByRole("button", { name: "Close" }));
    expect(screen.queryByText(message)).not.toBeInTheDocument();
    unmount();

    render(<EarlyAccessBanner />);
    expect(screen.queryByText(message)).not.toBeInTheDocument();
  });
});
