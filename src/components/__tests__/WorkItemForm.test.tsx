import { describe, it, expect, vi } from 'vitest';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import WorkItemForm from '../WorkItemForm';
import type { WorkItem } from '../../types/domain';

const noop = vi.fn();

describe('WorkItemForm', () => {
  it('renders the create-mode title when no editingItem is provided', () => {
    render(
      <WorkItemForm isOpen={true} onClose={noop} onSubmit={noop} editingItem={null} />,
    );
    expect(screen.getByText('新建工作项')).toBeInTheDocument();
  });

  it('renders empty fields in create mode', () => {
    render(
      <WorkItemForm isOpen={true} onClose={noop} onSubmit={noop} editingItem={null} />,
    );
    const titleInput = screen.getByPlaceholderText('请输入标题') as HTMLInputElement;
    expect(titleInput.value).toBe('');
  });

  it('renders the edit-mode title and pre-fills fields', () => {
    const item: WorkItem = {
      id: '1',
      title: 'Existing',
      content: 'Some content',
      status: 'inProgress',
      priority: 'high',
      source: 'email',
      tags: ['a', 'b'],
      createdAt: 1700000000,
      updatedAt: 1700000000,
      parentId: null,
      latitude: null,
      longitude: null,
      gisFeatureId: null,
    };

    render(
      <WorkItemForm isOpen={true} onClose={noop} onSubmit={noop} editingItem={item} />,
    );
    expect(screen.getByText('编辑工作项')).toBeInTheDocument();
    const titleInput = screen.getByPlaceholderText('请输入标题') as HTMLInputElement;
    expect(titleInput.value).toBe('Existing');
  });

  it('submits with correct data when the save button is clicked', async () => {
    const onSubmit = vi.fn().mockResolvedValue(undefined);
    const onClose = vi.fn();

    render(
      <WorkItemForm isOpen={true} onClose={onClose} onSubmit={onSubmit} editingItem={null} />,
    );

    fireEvent.change(screen.getByPlaceholderText('请输入标题'), {
      target: { value: 'New item' },
    });

    fireEvent.click(screen.getByRole('button', { name: '保存' }));

    await waitFor(() => {
      expect(onSubmit).toHaveBeenCalledOnce();
    });

    const submitted = onSubmit.mock.calls[0][0] as WorkItem;
    expect(submitted.title).toBe('New item');
    expect(submitted.status).toBe('new');
    expect(submitted.priority).toBe('normal');
  });

  it('disables save button when title is empty', () => {
    render(
      <WorkItemForm isOpen={true} onClose={noop} onSubmit={noop} editingItem={null} />,
    );
    const saveBtn = screen.getByRole('button', { name: '保存' });
    expect(saveBtn).toBeDisabled();
  });
});
