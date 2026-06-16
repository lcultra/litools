import { invokeSdk } from './runtime';
import type { DynamicCommand } from './types';

// ---- commands API ----

export function add(commands: DynamicCommand[]): Promise<void> {
  return invokeSdk('sdk_commands_add', { commands });
}

export function remove(ids: string[]): Promise<void> {
  return invokeSdk('sdk_commands_remove', { ids });
}

export function replace(commands: DynamicCommand[]): Promise<void> {
  return invokeSdk('sdk_commands_replace', { commands });
}

export function update(id: string, cmd: Partial<DynamicCommand>): Promise<void> {
  return invokeSdk('sdk_commands_update', { id, cmd });
}
