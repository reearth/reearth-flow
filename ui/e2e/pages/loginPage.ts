import { Locator, Page } from "@playwright/test";

export class LoginPage {
  readonly emailInput: Locator;
  readonly passwordInput: Locator;
  readonly continueButton: Locator;
  readonly loginButton: Locator;
  readonly errorMessage: Locator;
  readonly forgotPasswordLink: Locator;
  readonly editEmailLink: Locator;
  readonly loggedInMarker: Locator;

  constructor(page: Page) {
    this.emailInput = page.locator("input#username");
    this.passwordInput = page.locator("input#password");
    this.continueButton = page.getByRole("button", { name: "Continue" });
    this.loginButton = page.getByRole("button", { name: "Log In" });
    this.errorMessage = page.locator("#error-element-password");
    this.forgotPasswordLink = page.getByRole("link", {
      name: /Don't remember your password/i,
    });
    this.editEmailLink = page.getByRole("link", { name: "Edit" });
    this.loggedInMarker = page.getByText("Re:Earth Flow", { exact: true });
  }

  async isLoggedIn() {
    return this.loggedInMarker.isVisible().catch(() => false);
  }

  async login(email: string, password: string) {
    await this.emailInput.waitFor({ state: "visible" });
    await this.emailInput.fill(email);
    await this.continueButton.first().click();

    await this.passwordInput.waitFor({ state: "visible" });
    await this.passwordInput.fill(password);
    await this.continueButton.click();
  }

  async waitForLoggedIn() {
    await this.loggedInMarker.waitFor({ state: "visible", timeout: 45_000 });
  }
}
