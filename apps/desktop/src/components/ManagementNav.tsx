import { A } from '@solidjs/router';
import { For, Show } from 'solid-js';
import { type ManagementNavGroup, managementNavGroupList, managementNavItems } from '../views/registry';

export function ManagementNav() {
    function itemsForGroup(group: ManagementNavGroup) {
        return managementNavItems.filter((item) => item.navGroup === group.id);
    }

    return (
        <aside class="flex h-full w-48 shrink-0 flex-col border-border border-r px-3 py-4">
            <nav class="grid gap-4 text-sm">
                <For each={managementNavGroupList}>
                    {(group) => (
                        <Show when={itemsForGroup(group).length}>
                            <div class="grid gap-1">
                                <p class="m-0 px-3 text-xs font-semibold text-muted">{group.label}</p>
                                <For each={itemsForGroup(group)}>
                                    {(item) => (
                                        <A
                                            class="rounded-lg px-3 py-2 text-muted outline-none transition-colors hover:bg-surface-muted/70 hover:text-fg focus-visible:bg-surface-muted/70 focus-visible:text-fg"
                                            activeClass="bg-surface-muted text-fg font-semibold"
                                            href={item.path}
                                            inactiveClass=""
                                            end
                                        >
                                            {item.label}
                                        </A>
                                    )}
                                </For>
                            </div>
                        </Show>
                    )}
                </For>
            </nav>
        </aside>
    );
}
