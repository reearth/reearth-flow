"use client";

import * as DialogPrimitive from "@radix-ui/react-dialog";
import { Cross2Icon } from "@radix-ui/react-icons";
import { forwardRef, ForwardRefExoticComponent, RefAttributes } from "react";

import { cn } from "@flow/lib/utils";

const Dialog = DialogPrimitive.Root;

const DialogTrigger = DialogPrimitive.Trigger;

const DialogPortal = DialogPrimitive.Portal;

const DialogClose = DialogPrimitive.Close;

const DialogOverlay = forwardRef<
  React.ElementRef<typeof DialogPrimitive.Overlay>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Overlay> & {
    overlayBgClass?: string;
  }
>(({ className, overlayBgClass = "bg-black/40", ...props }, ref) => (
  <DialogPrimitive.Overlay
    ref={ref}
    className={cn(
      overlayBgClass,
      "fixed inset-0 z-50 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:animate-in data-[state=open]:fade-in-0",
      className,
    )}
    {...props}
  />
));
DialogOverlay.displayName = DialogPrimitive.Overlay.displayName;

const DialogContent = forwardRef<
  React.ElementRef<typeof DialogPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content> & {
    size?: "xs" | "sm" | "md" | "lg" | "xl" | "2xl" | "3xl" | "full";
    position?: "center" | "off-center" | "top";
    overlayBgClass?: string;
    hideCloseButton?: boolean;
  }
>(
  (
    {
      className,
      children,
      size,
      position = "center",
      overlayBgClass,
      hideCloseButton,
      onOpenAutoFocus,
      onCloseAutoFocus,
      ...props
    },
    ref,
  ) => (
    <DialogPortal>
      <DialogOverlay overlayBgClass={overlayBgClass} />
      <DialogPrimitive.Content
        ref={ref}
        className={cn(
          "fixed top-[50%] left-[50%] z-50 grid w-full max-w-xl translate-x-[-50%] gap-4 border border-primary bg-card/50 shadow-lg backdrop-blur duration-200 data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=closed]:zoom-out-95 data-[state=open]:animate-in data-[state=open]:fade-in-0 data-[state=open]:zoom-in-95 sm:rounded-lg",
          size === "xs"
            ? "max-w-[300px]"
            : size === "sm"
              ? "max-w-[400px]"
              : size === "md"
                ? "max-w-[500px]"
                : size === "lg"
                  ? "max-w-[600px]"
                  : size === "xl"
                    ? "max-w-[700px]"
                    : size === "2xl"
                      ? "max-w-[900px]"
                      : size === "3xl"
                        ? "max-w-[1000px]"
                        : undefined,
          position === "top"
            ? "top-[15%]"
            : position === "off-center"
              ? "top-[40%] translate-y-[-50%]"
              : position === "center"
                ? "top-[50%] translate-y-[-50%]"
                : undefined,
          size === "full"
            ? "top-0 right-0 bottom-0 left-0 max-w-full translate-0"
            : undefined,
          className,
        )}
        onOpenAutoFocus={(e) =>
          onOpenAutoFocus ? onOpenAutoFocus(e) : e.preventDefault()
        }
        onCloseAutoFocus={(e) =>
          onCloseAutoFocus ? onCloseAutoFocus(e) : e.preventDefault()
        }
        {...props}>
        <div className="overflow-hidden rounded-lg">
          {children}
          {!hideCloseButton && (
            <DialogPrimitive.Close className="absolute top-4 right-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted-foreground">
              <Cross2Icon className="size-5" />
              <span className="sr-only">Close</span>
            </DialogPrimitive.Close>
          )}
        </div>
      </DialogPrimitive.Content>
    </DialogPortal>
  ),
);
DialogContent.displayName = DialogPrimitive.Content.displayName;

const DialogHeader = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn(
      "flex flex-col space-y-1.5 text-center sm:text-left",
      className,
    )}
    {...props}
  />
);
DialogHeader.displayName = "DialogHeader";

const DialogFooter = ({
  className,
  ...props
}: React.HTMLAttributes<HTMLDivElement>) => (
  <div
    className={cn(
      "flex flex-col-reverse px-4 pb-4 sm:flex-row sm:justify-end sm:space-x-2",
      className,
    )}
    {...props}
  />
);
DialogFooter.displayName = "DialogFooter";

const DialogTitle = forwardRef<
  React.ElementRef<typeof DialogPrimitive.Title>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Title>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Title
    ref={ref}
    className={cn(
      "rounded-t-lg p-4 text-xl leading-none tracking-tight dark:font-thin",
      className,
    )}
    {...props}
  />
));
DialogTitle.displayName = DialogPrimitive.Title.displayName;

const DialogDescription = forwardRef<
  React.ElementRef<typeof DialogPrimitive.Description>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Description>
>(({ className, ...props }, ref) => (
  <DialogPrimitive.Description
    ref={ref}
    className={cn("px-4 text-sm", className)}
    {...props}
  />
));
DialogDescription.displayName = DialogPrimitive.Description.displayName;

const DialogContentWrapper = forwardRef<
  React.ElementRef<
    ForwardRefExoticComponent<
      {
        className?: string;
        children?: React.ReactNode;
      } & RefAttributes<HTMLDivElement>
    >
  >,
  React.ComponentPropsWithoutRef<
    ForwardRefExoticComponent<
      {
        className?: string;
        children?: React.ReactNode;
      } & RefAttributes<HTMLDivElement>
    >
  >
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("flex flex-col gap-4 overflow-hidden px-6 py-4", className)}
    {...props}
  />
));
DialogContentWrapper.displayName = "DialogContentWrapper";

const DialogContentSection = forwardRef<
  React.ElementRef<
    ForwardRefExoticComponent<
      {
        className?: string;
        children?: React.ReactNode;
      } & RefAttributes<HTMLDivElement>
    >
  >,
  React.ComponentPropsWithoutRef<
    ForwardRefExoticComponent<
      {
        className?: string;
        children?: React.ReactNode;
      } & RefAttributes<HTMLDivElement>
    >
  >
>(({ className, ...props }, ref) => (
  <div
    ref={ref}
    className={cn("flex flex-col gap-2 overflow-hidden", className)}
    {...props}
  />
));
DialogContentSection.displayName = "DialogContentSection";

export {
  Dialog,
  DialogPortal,
  DialogOverlay,
  DialogTrigger,
  DialogClose,
  DialogContent,
  DialogHeader,
  DialogFooter,
  DialogTitle,
  DialogDescription,
  DialogContentWrapper,
  DialogContentSection,
};
