import { invoke } from '@tauri-apps/api/core';
import type {
  WorkItem,
  Contact,
  AiProfile,
  MailAccount,
  GisLayer,
  TileSource,
  SearchResult,
} from '../types/domain';

// All IPC commands return JSON strings from Rust; we parse them here.

function parse<T>(json: string): T {
  return JSON.parse(json) as T;
}

// ── Work Items ───────────────────────────────────────────────────────────────

export async function listWorkItems(): Promise<WorkItem[]> {
  const json = await invoke<string>('list_work_items');
  return parse<WorkItem[]>(json);
}

export async function getWorkItem(id: string): Promise<WorkItem | null> {
  const json = await invoke<string>('get_work_item', { id });
  return parse<WorkItem | null>(json);
}

export async function createWorkItem(item: WorkItem): Promise<WorkItem> {
  const json = await invoke<string>('create_work_item', { json: JSON.stringify(item) });
  return parse<WorkItem>(json);
}

export async function updateWorkItem(item: WorkItem): Promise<WorkItem> {
  const json = await invoke<string>('update_work_item', { json: JSON.stringify(item) });
  return parse<WorkItem>(json);
}

export async function deleteWorkItem(id: string): Promise<void> {
  await invoke<void>('delete_work_item', { id });
}

// ── Search ───────────────────────────────────────────────────────────────────

export async function search(query: string, limit?: number): Promise<SearchResult[]> {
  const json = await invoke<string>('search', { query, limit });
  return parse<SearchResult[]>(json);
}

// ── Contacts ─────────────────────────────────────────────────────────────────

export async function listContacts(): Promise<Contact[]> {
  const json = await invoke<string>('list_contacts');
  return parse<Contact[]>(json);
}

// ── AI ───────────────────────────────────────────────────────────────────────

export async function analyzeWorkItem(id: string, analysisType: string): Promise<string> {
  return invoke<string>('analyze_work_item', { id, analysisType });
}

export async function listAiProfiles(): Promise<AiProfile[]> {
  const json = await invoke<string>('list_ai_profiles');
  return parse<AiProfile[]>(json);
}

// ── GIS ──────────────────────────────────────────────────────────────────────

export async function listTileSources(): Promise<TileSource[]> {
  const json = await invoke<string>('list_tile_sources');
  return parse<TileSource[]>(json);
}

export async function listGisLayers(): Promise<GisLayer[]> {
  const json = await invoke<string>('list_gis_layers');
  return parse<GisLayer[]>(json);
}

// ── Mail ─────────────────────────────────────────────────────────────────────

export async function listMailAccounts(): Promise<MailAccount[]> {
  const json = await invoke<string>('list_mail_accounts');
  return parse<MailAccount[]>(json);
}
