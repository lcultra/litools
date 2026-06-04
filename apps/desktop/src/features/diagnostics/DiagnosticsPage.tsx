import { For, Show, createResource } from 'solid-js';
import { getDiagnostics } from '../../bridge/commands';

export function DiagnosticsPage() {
  const [diagnostics, { refetch }] = createResource(getDiagnostics);

  return (
    <section class="rounded-[20px] border border-current/10 bg-current/5 p-6 shadow-[0_24px_80px_rgba(0,0,0,0.18)]">
      <div class="flex items-start justify-between gap-4">
        <div>
          <h1 class="m-0 text-2xl font-semibold">Diagnostics</h1>
          <p class="m-0 mt-2 text-sm text-current/60">Runtime health and local state for litools.</p>
        </div>
        <button class="rounded-lg bg-current px-4 py-2 text-sm font-semibold text-white dark:text-[#111318]" onClick={() => void refetch()} type="button">
          Refresh
        </button>
      </div>

      <Show when={diagnostics()} fallback={<p class="mt-6 text-sm text-current/60">Loading diagnostics...</p>}>
        {(diagnostics) => (
          <>
            <div class="mt-6 grid gap-4 text-sm">
              <DiagnosticRow label="App version" value={diagnostics().app_version} />
              <DiagnosticRow label="Platform" value={diagnostics().platform} />
              <DiagnosticRow label="App data dir" value={diagnostics().app_data_dir} />
              <DiagnosticRow label="Installed plugins" value={String(diagnostics().plugin_count)} />
              <DiagnosticRow label="Indexed commands" value={String(diagnostics().command_count)} />
              <DiagnosticRow label="Recent usage count" value={String(diagnostics().recent_usage_count)} />
              <DiagnosticRow label="Theme" value={diagnostics().settings.theme} />
              <DiagnosticRow label="Result limit" value={String(diagnostics().settings.palette.result_limit)} />
              <DiagnosticRow label="Enabled providers" value={diagnostics().settings.search.enabled_providers.join(', ')} />
              <DiagnosticRow label="Window behavior" value={windowBehaviorSummary(diagnostics().settings)} />
              <DiagnosticRow label="Global hotkey" value={diagnostics().shortcut.accelerator} />
              <DiagnosticRow label="Shortcut status" value={diagnostics().shortcut.registered ? 'Registered' : diagnostics().shortcut.error ?? 'Not registered'} />
            </div>

            <div class="mt-8">
              <h2 class="m-0 text-lg font-semibold">Recent usage</h2>
              <div class="mt-3 grid gap-2 text-sm">
                <Show when={diagnostics().recent_usage.length} fallback={<p class="m-0 text-current/60">No usage recorded yet.</p>}>
                  <For each={diagnostics().recent_usage}>
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
          </>
        )}
      </Show>
    </section>
  );
}

function DiagnosticRow(props: { label: string; value: string }) {
  return (
    <div class="grid gap-1 rounded-xl bg-current/5 px-4 py-3 sm:flex sm:items-center sm:justify-between sm:gap-4">
      <span class="text-current/60">{props.label}</span>
      <span class="break-all font-medium">{props.value}</span>
    </div>
  );
}

function windowBehaviorSummary(settings: { window: { hide_on_blur: boolean; close_to_tray: boolean; center_on_show: boolean } }) {
  return [
    settings.window.hide_on_blur ? 'hide on blur' : 'stay on blur',
    settings.window.close_to_tray ? 'close to tray' : 'close exits',
    settings.window.center_on_show ? 'center on show' : 'keep position',
  ].join(', ');
}
