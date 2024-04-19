import { Pencil1Icon } from "@radix-ui/react-icons";
import { forwardRef } from "react";

import {
  Avatar,
  AvatarFallback,
  AvatarImage,
  NavigationMenuContent,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuTrigger,
} from "@flow/components";
import { cn } from "@flow/lib/utils";

const UserNavigation: React.FC = () => {
  return (
    <NavigationMenuItem className="flex-1">
      <NavigationMenuTrigger>
        <div className="flex gap-2 mr-2">
          <Avatar className="h-9 w-9">
            <AvatarImage src="https://www.gravatar.com/avatar/205e460b479e2e5b48aec07710c08d50" />
            <AvatarFallback>KW</AvatarFallback>
          </Avatar>
          <div className="self-center">
            <p className="text-zinc-400">kyle-01234</p>
          </div>
        </div>
      </NavigationMenuTrigger>
      <NavigationMenuContent>
        <ul className="grid gap-3 p-4 md:w-[400px] lg:w-[500px] lg:grid-cols-[.75fr_1fr]">
          <li className="row-span-3">
            <NavigationMenuLink asChild>
              <a
                className="flex h-full w-full select-none flex-col justify-end rounded-md bg-gradient-to-b from-muted/50 to-muted p-6 no-underline outline-none focus:shadow-md"
                href="/">
                <Pencil1Icon className="h-6 w-6" />
                <div className="mb-2 mt-4 text-lg font-medium">shadcn/ui</div>
                <p className="text-sm leading-tight text-muted-foreground">
                  Beautifully designed components built with Radix UI and Tailwind CSS.
                </p>
              </a>
            </NavigationMenuLink>
          </li>
          <ListItem href="/docs" title="Account settings">
            Update info connected to your account, like UserNavigationname, email and password.
          </ListItem>
          <ListItem href="/docs/installation" title="Installation">
            How to install dependencies and structure your app.
          </ListItem>
          <ListItem href="/docs/primitives/typography" title="Typography">
            Styles for headings, paragraphs, lists...etc
          </ListItem>
        </ul>
      </NavigationMenuContent>
    </NavigationMenuItem>
  );
};

export { UserNavigation };

const ListItem = forwardRef<React.ElementRef<"a">, React.ComponentPropsWithoutRef<"a">>(
  ({ className, title, children, ...props }, ref) => {
    return (
      <li>
        <NavigationMenuLink asChild>
          <a
            ref={ref}
            className={cn(
              "block select-none space-y-1 rounded-md p-3 leading-none no-underline outline-none transition-colors hover:bg-zinc-800 focus:bg-zinc-800",
              className,
            )}
            {...props}>
            <div className="text-sm font-medium leading-none">{title}</div>
            <p className="line-clamp-2 text-sm leading-snug text-muted-foreground">{children}</p>
          </a>
        </NavigationMenuLink>
      </li>
    );
  },
);

ListItem.displayName = "ListItem";
