import { useLocation } from '@solidjs/router';
import { X } from 'lucide-solid';
import { startDragging } from '../bridge/commands';
import { routeForPath } from '../views/registry';

type ManagementHeaderProps = {
    onClose: () => void;
};

export function ManagementHeader(props: ManagementHeaderProps) {
    const location = useLocation();
    const currentRoute = () => routeForPath(location.pathname);

    function handleDragPointerDown(event: PointerEvent) {
        if (event.button !== 0) {
            return;
        }

        void startDragging();
    }

    return (
        <header class="flex h-[68px] shrink-0 items-center gap-2 border-border border-b px-3">
            <div class="flex items-center overflow-hidden rounded-full border border-border bg-surface-muted text-sm">
                <div class="flex items-center gap-2 py-1.5 pl-3 pr-2" onPointerDown={handleDragPointerDown}>
                    <span class="font-semibold text-fg">管理</span>
                    <span class="text-muted">/</span>
                    <span class="text-muted">{currentRoute().label}</span>
                </div>
                <button
                    aria-label="关闭管理面板"
                    class="grid size-8 cursor-pointer place-items-center border-border border-l text-muted outline-none transition-colors hover:bg-danger/10 hover:text-danger focus-visible:ring-2 focus-visible:ring-accent/30 focus-visible:outline-none"
                    onClick={props.onClose}
                    type="button"
                >
                    <X size={16} strokeWidth={2} />
                </button>
            </div>
            <div aria-hidden="true" class="min-w-0 flex-1 self-stretch cursor-grab active:cursor-grabbing" onPointerDown={handleDragPointerDown} />
        </header>
    );
}
