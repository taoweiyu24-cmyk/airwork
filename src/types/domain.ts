// TypeScript interfaces mirroring Rust domain entities.
// All field names use camelCase matching the Rust `#[serde(rename_all = "camelCase")]`.

// ── Enums ────────────────────────────────────────────────────────────────────

export type WorkItemStatus = 'new' | 'inProgress' | 'reviewing' | 'done' | 'archived';
export type Priority = 'low' | 'normal' | 'high' | 'urgent';
export type Source = 'manual' | 'email' | 'file' | 'api';
export type AnalysisType = 'summary' | 'actionExtraction' | 'classification' | 'prioritySuggestion';
export type LayerType = 'vector' | 'raster' | 'tile';
export type GeometryType = 'point' | 'lineString' | 'polygon' | 'multiPoint' | 'multiLineString' | 'multiPolygon';

// ── Entities ─────────────────────────────────────────────────────────────────

export interface WorkItem {
  id: string;
  title: string;
  content: string | null;
  status: WorkItemStatus;
  priority: Priority;
  source: Source;
  tags: string[];
  createdAt: number;
  updatedAt: number;
  parentId: string | null;
  latitude: number | null;
  longitude: number | null;
  gisFeatureId: string | null;
}

export interface Contact {
  id: string;
  name: string;
  email: string | null;
  phone: string | null;
  organization: string | null;
  createdAt: number;
}

export interface MailAccount {
  id: string;
  name: string;
  email: string;
  provider: string;
  imapHost: string;
  imapPort: number;
  smtpHost: string;
  smtpPort: number;
  useOauth: boolean;
  accessToken: string | null;
  refreshToken: string | null;
  isActive: boolean;
}

export interface AiProfile {
  id: string;
  name: string;
  provider: string;
  apiKey: string;
  model: string;
  baseUrl: string;
  isDefault: boolean;
  maxTokens: number | null;
  temperature: number | null;
}

export interface GisLayer {
  id: string;
  name: string;
  layerType: LayerType;
  sourcePath: string | null;
  visible: boolean;
  opacity: number;
  styleJson: string | null;
  createdAt: number;
}

export interface TileSource {
  name: string;
  url_template: string;
  attribution: string;
  min_zoom: number;
  max_zoom: number;
}

export interface MailMessage {
  id: string;
  accountId: string;
  messageId: string;
  subject: string;
  fromAddress: string;
  toAddresses: string[];
  bodyText: string | null;
  bodyHtml: string | null;
  receivedAt: number;
  isRead: boolean;
  workItemId: string | null;
}

export interface SearchResult {
  id: string;
  title: string;
  snippet: string | null;
  score: number;
}
