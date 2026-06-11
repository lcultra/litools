import { Dialog } from '@ark-ui/solid';
import { AlertTriangle } from 'lucide-solid';
import { Button } from './Button';

type ConfirmDialogProps = {
    open: boolean;
    onClose: () => void;
    onConfirm: () => void;
    title: string;
    message: string;
    confirmLabel?: string;
    variant?: 'danger' | 'primary';
};

export function ConfirmDialog(props: ConfirmDialogProps) {
    return (
        <Dialog.Root open={props.open} onOpenChange={(e) => { if (!e.open) props.onClose(); }}>
            <Dialog.Backdrop class="fixed inset-0 bg-black/50 z-50" />
            <Dialog.Positioner class="fixed inset-0 flex items-center justify-center z-50">
                <Dialog.Content class="bg-bg rounded-lg border border-border p-6 max-w-sm w-full mx-4 shadow-xl">
                    <div class="flex items-start gap-3 mb-4">
                        {props.variant === 'danger' && (
                            <AlertTriangle class="w-5 h-5 text-red-500 shrink-0 mt-0.5" />
                        )}
                        <div>
                            <Dialog.Title class="text-base font-semibold">{props.title}</Dialog.Title>
                            <Dialog.Description class="text-sm text-fg-muted mt-1">{props.message}</Dialog.Description>
                        </div>
                    </div>
                    <div class="flex justify-end gap-2">
                        <Button variant="ghost" onClick={props.onClose}>取消</Button>
                        <Button variant={props.variant ?? 'primary'} onClick={props.onConfirm}>
                            {props.confirmLabel ?? '确认'}
                        </Button>
                    </div>
                </Dialog.Content>
            </Dialog.Positioner>
        </Dialog.Root>
    );
}
