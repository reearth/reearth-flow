import { Role } from "./member";
import { User } from "./user";

export type IntegrationMember = {
  id: string;
  integration?: Integration;
  integrationRole: Role;
  invitedById: string;
  active: boolean;
};

export type Integration = {
  id: string;
  name: string;
  description?: string;
  logoUrl: string;
  developerId: string;
  developer: User; // Developer like cms?
  iType: IntegrationType;
  config: {
    token?: string;
    // webhooks?: Webhook[]; // Prob don't need webhooks
  };
};

type IntegrationType = "Private" | "Public";
