// Тонкая обёртка над Tauri invoke: не роняет UI при ошибке команды,
// а возвращает fallback и логирует проблему.
import { invoke } from "@tauri-apps/api/core";
import { fallbackState, type AppState } from "./types";

export async function invokeCommand<T>(
  name: string,
  args: Record<string, unknown> = {},
  fallback: T,
): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (error) {
    console.error(name, error);
    return fallback;
  }
}

export function getAppState(): Promise<AppState> {
  return invokeCommand<AppState>("get_app_state", {}, fallbackState);
}
