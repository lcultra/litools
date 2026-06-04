import { createResource } from 'solid-js';
import { getSettings } from '../../bridge/commands';

export function SettingsPage() {
  const [settings] = createResource(getSettings);

  return (
    <section class="rounded-[20px] border border-current/10 bg-current/5 p-6 shadow-[0_24px_80px_rgba(0,0,0,0.18)]">
      <h1 class="m-0 text-2xl font-semibold">Settings</h1>
      <div class="mt-6 grid gap-4 text-sm">
        <SettingRow label="Theme" value={settings()?.theme ?? 'Loading...'} />
        <SettingRow label="Global hotkey" value={settings()?.palette.global_hotkey ?? 'Loading...'} />
        <SettingRow label="Result limit" value={String(settings()?.palette.result_limit ?? 'Loading...')} />
        <SettingRow label="Enabled providers" value={settings()?.search.enabled_providers.join(', ') ?? 'Loading...'} />
      </div>
    </section>
  );
}

function SettingRow(props: { label: string; value: string }) {
  return (
    <div class="flex items-center justify-between gap-4 rounded-xl bg-current/5 px-4 py-3">
      <span class="text-current/60">{props.label}</span>
      <span class="font-medium">{props.value}</span>
    </div>
  );
}
