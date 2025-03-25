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
      "fixed inset-0 z-50 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0",
      className,
    )}
    {...props}
  />
));
DialogOverlay.displayName = DialogPrimitive.Overlay.displayName;

const DialogContent = forwardRef<
  React.ElementRef<typeof DialogPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Content> & {
    size?: "xs" | "sm" | "md" | "lg" | "xl" | "2xl";
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
          "fixed left-[50%] top-[50%] z-50 grid w-full max-w-xl translate-x-[-50%] gap-4 border shadow-lg duration-200 data-[state=open]:animate-in data-[state=closed]:animate-out data-[state=closed]:fade-out-0 data-[state=open]:fade-in-0 data-[state=closed]:zoom-out-95 data-[state=open]:zoom-in-95 data-[state=closed]:slide-out-to-left-1/2 data-[state=closed]:slide-out-to-top-[48%] data-[state=open]:slide-in-from-left-1/2 data-[state=open]:slide-in-from-top-[48%] sm:rounded-lg",
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
                      : undefined,
          position === "top"
            ? "top-[5%]"
            : position === "off-center"
              ? "top-[40%] translate-y-[-50%]"
              : position === "center"
                ? "top-[50%] translate-y-[-50%]"
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
        <div className="overflow-hidden rounded-lg bg-secondary">
          {children}
          {!hideCloseButton && (
            <DialogPrimitive.Close className="absolute right-4 top-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:outline-hidden focus:ring-2 focus:ring-ring focus:ring-offset-2 disabled:pointer-events-none data-[state=open]:bg-accent data-[state=open]:text-muted-foreground">
              <Cross2Icon className="size-4" />
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
      "flex flex-col-reverse px-6 pb-6 sm:flex-row sm:justify-end sm:space-x-2",
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
      "text-xl text-center dark:font-thin border-b leading-none tracking-tight px-6 py-4 rounded-t-lg",
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
    className={cn("text-sm", className)}
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
    className={cn("py-4 px-6 flex flex-col gap-4 overflow-hidden", className)}
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
