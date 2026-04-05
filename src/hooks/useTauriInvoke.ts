import { invoke } from '@tauri-apps/api/core';

export function useTauriInvoke() {
  async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
    return invoke<T>(command, args);
  }
  return { invoke: call };
}
