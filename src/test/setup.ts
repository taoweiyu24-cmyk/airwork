import '@testing-library/jest-dom/vitest';
import { vi } from 'vitest';

// Mock @tauri-apps/api/core globally so tests run outside the Tauri shell.
vi.mock('@tauri-apps/api/core', () => ({
  invoke: vi.fn(),
}));
