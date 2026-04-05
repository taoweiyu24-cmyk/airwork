import { describe, it, expect, vi, beforeEach } from 'vitest';
import { invoke } from '@tauri-apps/api/core';
import {
  listWorkItems,
  createWorkItem,
  search,
  deleteWorkItem,
} from '../api';
import type { WorkItem } from '../../types/domain';

const mockInvoke = vi.mocked(invoke);

beforeEach(() => {
  mockInvoke.mockReset();
});

const sampleItem: WorkItem = {
  id: '01ARZ3NDEKTSV4RRFFQ69G5FAV',
  title: 'Test',
  content: null,
  status: 'new',
  priority: 'normal',
  source: 'manual',
  tags: [],
  createdAt: 1700000000,
  updatedAt: 1700000000,
  parentId: null,
  latitude: null,
  longitude: null,
  gisFeatureId: null,
};

describe('listWorkItems', () => {
  it('parses the JSON string returned by invoke', async () => {
    mockInvoke.mockResolvedValue(JSON.stringify([sampleItem]));

    const items = await listWorkItems();

    expect(mockInvoke).toHaveBeenCalledWith('list_work_items');
    expect(items).toEqual([sampleItem]);
  });
});

describe('createWorkItem', () => {
  it('sends the serialised work item to invoke', async () => {
    mockInvoke.mockResolvedValue(JSON.stringify(sampleItem));

    const result = await createWorkItem(sampleItem);

    expect(mockInvoke).toHaveBeenCalledWith('create_work_item', {
      json: JSON.stringify(sampleItem),
    });
    expect(result).toEqual(sampleItem);
  });
});

describe('search', () => {
  it('passes query and limit to invoke', async () => {
    mockInvoke.mockResolvedValue(JSON.stringify([]));

    await search('keyword', 10);

    expect(mockInvoke).toHaveBeenCalledWith('search', {
      query: 'keyword',
      limit: 10,
    });
  });
});

describe('deleteWorkItem', () => {
  it('calls invoke with the correct id', async () => {
    mockInvoke.mockResolvedValue(undefined);

    await deleteWorkItem('abc123');

    expect(mockInvoke).toHaveBeenCalledWith('delete_work_item', { id: 'abc123' });
  });
});

describe('error handling', () => {
  it('rejects when invoke throws', async () => {
    mockInvoke.mockRejectedValue(new Error('IPC failure'));

    await expect(listWorkItems()).rejects.toThrow('IPC failure');
  });
});
