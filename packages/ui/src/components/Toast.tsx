import { Portal } from 'solid-js/web';
import { Toast, toaster } from '@kobalte/core/toast';
import { X, Info, CircleCheck, TriangleAlert, CircleX } from 'lucide-solid';

type ToastVariant = 'info' | 'success' | 'warning' | 'error';

interface ToastOptions {
  description: string;
  variant?: ToastVariant;
  duration?: number;
}

const variantClass: Record<ToastVariant, string> = {
  info: 'border-primary',
  success: 'border-success',
  warning: 'border-warning',
  error: 'border-danger',
};

const iconMap: Record<ToastVariant, typeof Info> = {
  info: Info,
  success: CircleCheck,
  warning: TriangleAlert,
  error: CircleX,
};

const iconColor: Record<ToastVariant, string> = {
  info: 'text-primary',
  success: 'text-success',
  warning: 'text-warning',
  error: 'text-danger',
};

export function ToastProvider() {
  return (
    <Portal>
      <Toast.Region
        duration={5000}
        limit={5}
        class="fixed bottom-4 right-4 z-9999 flex flex-col gap-2"
      >
        <Toast.List class="flex flex-col gap-2" />
      </Toast.Region>
    </Portal>
  );
}

export function toast(options: ToastOptions) {
  const variant = options.variant ?? 'info';
  const Icon = iconMap[variant];
  toaster.show(
    (props) => (

      <Toast toastId={props.toastId} duration={options.duration}>
        <div
          class={`flex items-start gap-2 bg-surface-card border-l-4 rounded-lg p-3 shadow-lg min-w-70 ${variantClass[variant]}`}
        >
          <span class={`mt-0.5 shrink-0 ${iconColor[variant]}`}>
            <Icon class="w-4 h-4" />
          </span>
          <Toast.Description class="text-sm text-text flex-1">
            {options.description}
          </Toast.Description>
          <Toast.CloseButton class="text-text-muted hover:text-text shrink-0 cursor-pointer">
            <X class="w-4 h-4" />
          </Toast.CloseButton>
        </div>
      </Toast>
    ),
  );
}

toast.error = (description: string) => toast({ description, variant: 'error' });
toast.success = (description: string) => toast({ description, variant: 'success' });
toast.warning = (description: string) => toast({ description, variant: 'warning' });
toast.info = (description: string) => toast({ description, variant: 'info' });
