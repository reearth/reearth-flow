"use client";

import { Dialog as DialogPrimitive } from "@base-ui/react/dialog";
import { XIcon } from "@phosphor-icons/react";
import { forwardRef, ForwardRefExoticComponent, RefAttributes } from "react";

import { cn } from "@flow/lib/utils";

const Dialog = DialogPrimitive.Root;

const DialogTrigger = DialogPrimitive.Trigger;

const DialogPortal = DialogPrimitive.Portal;

const DialogClose = DialogPrimitive.Close;

const DialogOverlay = forwardRef<
  React.ElementRef<typeof DialogPrimitive.Backdrop>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Backdrop> & {
    overlayBgClass?: string;
  }
>(
  (
    {
      className,
      overlayBgClass = "bg-zinc-300/40 dark:bg-black/40 terminal:bg-black/40 midnight:bg-zinc-900/40 synthwave:bg-purple-900/40",
      ...props
    },
    ref,
  ) => (
    <DialogPrimitive.Backdrop
      ref={ref}
      className={cn(
        overlayBgClass,
        "fixed inset-0 z-50 transition-opacity duration-200 data-ending-style:opacity-0 data-starting-style:opacity-0",
        className,
      )}
      {...props}
    />
  ),
);
DialogOverlay.displayName = "DialogOverlay";

const DialogContent = forwardRef<
  React.ElementRef<typeof DialogPrimitive.Popup>,
  React.ComponentPropsWithoutRef<typeof DialogPrimitive.Popup> & {
    size?: "xs" | "sm" | "md" | "lg" | "xl" | "2xl" | "3xl" | "4xl" | "full";
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
      ...props
    },
    ref,
  ) => (
    <DialogPortal>
      <DialogOverlay overlayBgClass={overlayBgClass} />
      <DialogPrimitive.Popup
        ref={ref}
        data-slot="dialog-content"
        initialFocus={false}
        finalFocus={false}
        className={cn(
          "fixed top-[50%] left-[50%] z-50 grid w-full max-w-xl translate-x-[-50%] gap-4 border border-accent bg-card/50 shadow-lg backdrop-blur transition-all duration-200 data-ending-style:scale-95 data-ending-style:opacity-0 data-starting-style:scale-95 data-starting-style:opacity-0 sm:rounded-lg dark:border-primary dark:bg-card/50",
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
                        : size === "4xl"
                          ? "max-w-[1200px]"
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
        {...props}>
        <div className="flex h-full flex-col overflow-hidden rounded-lg">
          {children}
          {!hideCloseButton && (
            <DialogPrimitive.Close className="absolute top-4 right-4 rounded-sm opacity-70 ring-offset-background transition-opacity hover:opacity-100 focus:ring-2 focus:ring-ring focus:ring-offset-2 focus:outline-hidden disabled:pointer-events-none">
              <XIcon className="size-5" />
              <span className="sr-only">Close</span>
            </DialogPrimitive.Close>
          )}
        </div>
      </DialogPrimitive.Popup>
    </DialogPortal>
  ),
);
DialogContent.displayName = "DialogContent";

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
      "rounded-t-lg p-4 text-xl leading-none font-light tracking-tight dark:font-thin",
      className,
    )}
    {...props}
  />
));
DialogTitle.displayName = "DialogTitle";

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
DialogDescription.displayName = "DialogDescription";

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
