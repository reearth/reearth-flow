"use client";

import { Toast } from "@base-ui/react/toast";
import type * as React from "react";

export type ToastVariant = "default" | "destructive" | "warning";

export type ToastData = { variant?: ToastVariant };

// A single global manager so `toast()` works from anywhere (including
// non-component modules like the gql hooks). `<ToastProvider toastManager>`
// in NotificationSystem renders from this same instance.
export const toastManager = Toast.createToastManager<ToastData>();

type ToastInput = {
  title?: React.ReactNode;
  description?: React.ReactNode;
  variant?: ToastVariant;
};

function toast({ title, description, variant = "default" }: ToastInput) {
  const id = toastManager.add({ title, description, data: { variant } });

  return {
    id,
    dismiss: () => toastManager.close(id),
    update: (props: ToastInput) =>
      toastManager.update(id, {
        title: props.title,
        description: props.description,
        ...(props.variant ? { data: { variant: props.variant } } : {}),
      }),
  };
}

function useToast() {
  return {
    toast,
    dismiss: (toastId?: string) => toastManager.close(toastId),
  };
}

export { useToast, toast };
