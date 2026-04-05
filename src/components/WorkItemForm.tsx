import { useState, useEffect, useCallback } from 'react';
import type { WorkItem, WorkItemStatus, Priority, Source } from '../types/domain';
import Modal from './ui/Modal';
import Input from './ui/Input';
import Button from './ui/Button';

// ── Label maps ──────────────────────────────────────────────────────────────

const priorityOptions: { value: Priority; label: string }[] = [
  { value: 'low', label: '低' },
  { value: 'normal', label: '普通' },
  { value: 'high', label: '高' },
  { value: 'urgent', label: '紧急' },
];

const sourceOptions: { value: Source; label: string }[] = [
  { value: 'manual', label: '手动' },
  { value: 'email', label: '邮件' },
  { value: 'file', label: '文件' },
  { value: 'api', label: 'API' },
];

// ── Types ───────────────────────────────────────────────────────────────────

interface WorkItemFormData {
  title: string;
  content: string;
  priority: Priority;
  source: Source;
  tags: string;
}

interface WorkItemFormProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (item: WorkItem) => Promise<void>;
  editingItem: WorkItem | null;
}

function makeEmptyForm(): WorkItemFormData {
  return { title: '', content: '', priority: 'normal', source: 'manual', tags: '' };
}

function formDataFromItem(item: WorkItem): WorkItemFormData {
  return {
    title: item.title,
    content: item.content ?? '',
    priority: item.priority,
    source: item.source,
    tags: item.tags.join(', '),
  };
}

function generateTempId(): string {
  return Date.now().toString(36) + Math.random().toString(36).slice(2);
}

// ── Select component (local) ────────────────────────────────────────────────

interface SelectProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
}

function Select({ label, value, onChange, options }: SelectProps) {
  return (
    <div className="flex flex-col gap-1">
      <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
        {label}
      </label>
      <select
        value={value}
        onChange={(e) => onChange(e.target.value)}
        className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500/20 dark:border-gray-600 dark:bg-gray-800 dark:text-white dark:focus:border-blue-400 dark:focus:ring-blue-400/20"
      >
        {options.map((o) => (
          <option key={o.value} value={o.value}>
            {o.label}
          </option>
        ))}
      </select>
    </div>
  );
}

// ── Textarea (local) ────────────────────────────────────────────────────────

interface TextareaProps {
  label: string;
  value: string;
  onChange: (value: string) => void;
  rows?: number;
}

function Textarea({ label, value, onChange, rows = 4 }: TextareaProps) {
  return (
    <div className="flex flex-col gap-1">
      <label className="text-sm font-medium text-gray-700 dark:text-gray-300">
        {label}
      </label>
      <textarea
        value={value}
        onChange={(e) => onChange(e.target.value)}
        rows={rows}
        className="rounded-lg border border-gray-300 bg-white px-3 py-2 text-sm text-gray-900 placeholder-gray-400 focus:border-blue-500 focus:outline-none focus:ring-2 focus:ring-blue-500/20 dark:border-gray-600 dark:bg-gray-800 dark:text-white dark:placeholder-gray-500 dark:focus:border-blue-400 dark:focus:ring-blue-400/20"
      />
    </div>
  );
}

// ── Main form component ─────────────────────────────────────────────────────

export default function WorkItemForm({ isOpen, onClose, onSubmit, editingItem }: WorkItemFormProps) {
  const [form, setForm] = useState<WorkItemFormData>(makeEmptyForm);
  const [submitting, setSubmitting] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setForm(editingItem ? formDataFromItem(editingItem) : makeEmptyForm());
    }
  }, [isOpen, editingItem]);

  const updateField = useCallback(
    <K extends keyof WorkItemFormData>(key: K, value: WorkItemFormData[K]) => {
      setForm((prev) => ({ ...prev, [key]: value }));
    },
    [],
  );

  const handleSubmit = useCallback(async () => {
    if (!form.title.trim()) return;
    setSubmitting(true);

    const now = Math.floor(Date.now() / 1000);
    const tags = form.tags
      .split(',')
      .map((t) => t.trim())
      .filter(Boolean);

    const item: WorkItem = {
      id: editingItem?.id ?? generateTempId(),
      title: form.title.trim(),
      content: form.content.trim() || null,
      status: editingItem?.status ?? ('new' as WorkItemStatus),
      priority: form.priority,
      source: form.source,
      tags,
      createdAt: editingItem?.createdAt ?? now,
      updatedAt: now,
      parentId: editingItem?.parentId ?? null,
      latitude: editingItem?.latitude ?? null,
      longitude: editingItem?.longitude ?? null,
      gisFeatureId: editingItem?.gisFeatureId ?? null,
    };

    try {
      await onSubmit(item);
      onClose();
    } finally {
      setSubmitting(false);
    }
  }, [form, editingItem, onSubmit, onClose]);

  const modalTitle = editingItem ? '编辑工作项' : '新建工作项';

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={modalTitle}>
      <div className="flex flex-col gap-4">
        <Input
          label="标题"
          placeholder="请输入标题"
          value={form.title}
          onChange={(e) => updateField('title', e.target.value)}
          required
        />

        <Textarea
          label="内容"
          value={form.content}
          onChange={(v) => updateField('content', v)}
        />

        <div className="grid grid-cols-2 gap-4">
          <Select
            label="优先级"
            value={form.priority}
            onChange={(v) => updateField('priority', v as Priority)}
            options={priorityOptions}
          />
          <Select
            label="来源"
            value={form.source}
            onChange={(v) => updateField('source', v as Source)}
            options={sourceOptions}
          />
        </div>

        <Input
          label="标签"
          placeholder="多个标签用逗号分隔"
          value={form.tags}
          onChange={(e) => updateField('tags', e.target.value)}
        />

        <div className="flex justify-end gap-3 pt-2">
          <Button variant="secondary" size="sm" onClick={onClose} disabled={submitting}>
            取消
          </Button>
          <Button
            variant="primary"
            size="sm"
            onClick={handleSubmit}
            disabled={submitting || !form.title.trim()}
          >
            {submitting ? '保存中...' : '保存'}
          </Button>
        </div>
      </div>
    </Modal>
  );
}
