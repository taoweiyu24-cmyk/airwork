import { useState } from 'react';
import Modal from '../ui/Modal';
import Input from '../ui/Input';
import Button from '../ui/Button';
import type { AiProfile } from '../../types/domain';

interface AiProfileFormProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (profile: AiProfile) => Promise<void>;
  initial: AiProfile | null;
}

function generateUlid(): string {
  const now = Date.now();
  const timeChars = '0123456789ABCDEFGHJKMNPQRSTVWXYZ';
  let time = '';
  let t = now;
  for (let i = 0; i < 10; i++) {
    time = timeChars[t % 32] + time;
    t = Math.floor(t / 32);
  }
  let random = '';
  for (let i = 0; i < 16; i++) {
    random += timeChars[Math.floor(Math.random() * 32)];
  }
  return time + random;
}

const defaultForm: AiProfile = {
  id: '',
  name: '',
  provider: 'openai',
  apiKey: '',
  model: '',
  baseUrl: 'https://api.openai.com/v1',
  isDefault: false,
  maxTokens: null,
  temperature: null,
};

export default function AiProfileForm({ isOpen, onClose, onSubmit, initial }: AiProfileFormProps) {
  const isEdit = initial !== null;
  const [form, setForm] = useState<AiProfile>(initial ?? { ...defaultForm, id: generateUlid() });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleChange = (field: keyof AiProfile, value: string | boolean | number | null) => {
    setForm((prev) => ({ ...prev, [field]: value }));
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    setError(null);
    try {
      await onSubmit(form);
      onClose();
    } catch (err: unknown) {
      const msg = err instanceof Error ? err.message : '保存失败';
      setError(msg);
    } finally {
      setSaving(false);
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title={isEdit ? '编辑 AI 配置' : '新建 AI 配置'}>
      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <Input
          label="名称"
          value={form.name}
          onChange={(e) => handleChange('name', e.target.value)}
          required
          placeholder="例如：GPT-4o"
        />
        <Input
          label="提供商"
          value={form.provider}
          onChange={(e) => handleChange('provider', e.target.value)}
          required
          placeholder="例如：openai"
        />
        <Input
          label="模型"
          value={form.model}
          onChange={(e) => handleChange('model', e.target.value)}
          required
          placeholder="例如：gpt-4o"
        />
        <Input
          label="API 密钥"
          type="password"
          value={form.apiKey}
          onChange={(e) => handleChange('apiKey', e.target.value)}
          required
          placeholder="sk-..."
        />
        <Input
          label="Base URL"
          value={form.baseUrl}
          onChange={(e) => handleChange('baseUrl', e.target.value)}
          required
          placeholder="https://api.openai.com/v1"
        />
        <div className="grid grid-cols-2 gap-4">
          <Input
            label="最大令牌数"
            type="number"
            value={form.maxTokens ?? ''}
            onChange={(e) => {
              const v = e.target.value;
              handleChange('maxTokens', v === '' ? null : parseInt(v, 10));
            }}
            placeholder="可选"
          />
          <Input
            label="温度"
            type="number"
            step="0.1"
            min="0"
            max="2"
            value={form.temperature ?? ''}
            onChange={(e) => {
              const v = e.target.value;
              handleChange('temperature', v === '' ? null : parseFloat(v));
            }}
            placeholder="可选，0-2"
          />
        </div>
        <label className="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
          <input
            type="checkbox"
            checked={form.isDefault}
            onChange={(e) => handleChange('isDefault', e.target.checked)}
            className="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
          />
          设为默认配置
        </label>

        {error && (
          <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
        )}

        <div className="flex justify-end gap-3 pt-2">
          <Button type="button" variant="secondary" onClick={onClose} disabled={saving}>
            取消
          </Button>
          <Button type="submit" disabled={saving}>
            {saving ? '保存中...' : isEdit ? '更新' : '创建'}
          </Button>
        </div>
      </form>
    </Modal>
  );
}
