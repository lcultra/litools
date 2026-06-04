import { For, Show, createResource } from 'solid-js';
import { getDiagnostics } from '../../bridge/commands';

export function DiagnosticsPage() {
  const [diagnostics] = createResource(getDiagnostics);

  return (
    <section class="rounded-[20px] border border-current/10 bg-current/5 p-6 shadow-[0_24px_80px_rgba(0,0,0,0.18)]">
      <h1 class="m-0 text-2xl font-semibold">Diagnostics</h1>
      <div class="mt-6 grid gap-4 text-sm">
        <DiagnosticRow label="App version" value={diagnostics()?.app_version ?? 'Loading...'} />
        <DiagnosticRow label="Installed plugins" value={String(diagnostics()?.plugin_count ?? 'Loading...')} />
      </div>

      <div class="mt-8">
        <h2 class="m-0 text-lg font-semibold">Recent usage</h2>
        <div class="mt-3 grid gap-2 text-sm">
          <Show when={diagnostics()?.recent_usage.length} fallback={<p class="m-0 text-current/60">No usage recorded yet.</p>}>
            <For each={diagnostics()?.recent_usage ?? []}>
              {(event) => (
                <div class="grid gap-1 rounded-xl bg-current/5 px-4 py-3">
                  <div class="flex items-center justify-between gap-4">
                    <span class="font-medium">{event.target_id}</span>
                    <span class="text-xs text-current/50">{event.target_type}</span>
                  </div>
                  <span class="text-xs text-current/50">{event.selected_at}</span>
                </div>
              )}
            </For>
          </Show>
        </div>
      </div>
    </section>
  );
}

function DiagnosticRow(props: { label: string; value: string }) {
  return (
    <div class="flex items-center justify-between gap-4 rounded-xl bg-current/5 px-4 py-3">
      <span class="text-current/60">{props.label}</span>
      <span class="font-medium">{props.value}</span>
    </div>
  );
}
