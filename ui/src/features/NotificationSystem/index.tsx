"use client";

import { Toast as ToastPrimitive } from "@base-ui/react/toast";

import {
  Toast,
  ToastClose,
  ToastDescription,
  ToastPortal,
  ToastProvider,
  ToastTitle,
  ToastViewport,
} from "@flow/components";

import { toastManager, type ToastData } from "./useToast";

export function NotificationSystem() {
  return (
    <ToastProvider toastManager={toastManager} limit={3}>
      <ToastPortal>
        <ToastViewport>
          <ToastList />
        </ToastViewport>
      </ToastPortal>
    </ToastProvider>
  );
}

function ToastList() {
  const { toasts } = ToastPrimitive.useToastManager();

  return (
    <>
      {toasts.map((toast) => (
        <Toast
          key={toast.id}
          toast={toast}
          variant={(toast.data as ToastData | undefined)?.variant}>
          <div className="grid gap-1">
            {toast.title && <ToastTitle>{toast.title}</ToastTitle>}
            {toast.description && (
              <ToastDescription>{toast.description}</ToastDescription>
            )}
          </div>
          <ToastClose />
        </Toast>
      ))}
    </>
  );
}
