import { Dialog } from '@kobalte/core/dialog';
import { TriangleAlert } from 'lucide-solid';
import { Button } from './Button';

interface ConfirmDialogProps {
  open: boolean;
  onClose: () => void;
  onConfirm: () => void;
  title: string;
  description: string;
  confirmLabel?: string;
  variant?: 'primary' | 'danger';
}

export function ConfirmDialog(props: ConfirmDialogProps) {
  return (
    <Dialog open={props.open} onOpenChange={(open) => { if (!open) props.onClose(); }}>
      <Dialog.Portal>
        <Dialog.Overlay class="fixed inset-0 bg-overlay z-50" />
        <Dialog.Content class="fixed inset-0 z-50 flex items-center justify-center p-4">
          <div class="bg-surface-card border border-border rounded-lg p-4 w-[calc(100%-2rem)] max-w-75 shadow-xl">
            <div class="flex items-start gap-3">
              {props.variant === 'danger' && (
                <div class="mt-0.5 text-danger">
                  <TriangleAlert class="w-5 h-5" />
                </div>
              )}
              <div class="flex-1">
                <Dialog.Title class="text-base font-semibold text-text">
                  {props.title}
                </Dialog.Title>
                <Dialog.Description class="mt-1 text-sm text-text-muted">
                  {props.description}
                </Dialog.Description>
              </div>
            </div>
            <div class="flex justify-end gap-2 mt-4">
              <Button variant="secondary" size="sm" onClick={props.onClose}>
                取消
              </Button>
              <Button
                variant={props.variant === 'danger' ? 'danger' : 'primary'}
                size="sm"
                onClick={props.onConfirm}
              >
                {props.confirmLabel ?? '确认'}
              </Button>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog>
  );
}
