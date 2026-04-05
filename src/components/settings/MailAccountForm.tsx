import { useState } from 'react';
import Modal from '../ui/Modal';
import Input from '../ui/Input';
import Button from '../ui/Button';
import type { MailAccount } from '../../types/domain';

interface MailAccountFormProps {
  isOpen: boolean;
  onClose: () => void;
  onSubmit: (account: MailAccount) => Promise<void>;
  initial: MailAccount | null;
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

const defaultForm: MailAccount = {
  id: '',
  name: '',
  email: '',
  provider: '',
  imapHost: '',
  imapPort: 993,
  smtpHost: '',
  smtpPort: 465,
  useOauth: false,
  accessToken: null,
  refreshToken: null,
  isActive: true,
};

export default function MailAccountForm({ isOpen, onClose, onSubmit, initial }: MailAccountFormProps) {
  const isEdit = initial !== null;
  const [form, setForm] = useState<MailAccount>(initial ?? { ...defaultForm, id: generateUlid() });
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleChange = (field: keyof MailAccount, value: string | boolean | number | null) => {
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
    <Modal isOpen={isOpen} onClose={onClose} title={isEdit ? '编辑邮箱账户' : '新建邮箱账户'}>
      <form onSubmit={handleSubmit} className="flex flex-col gap-4">
        <Input
          label="名称"
          value={form.name}
          onChange={(e) => handleChange('name', e.target.value)}
          required
          placeholder="例如：工作邮箱"
        />
        <Input
          label="邮箱地址"
          type="email"
          value={form.email}
          onChange={(e) => handleChange('email', e.target.value)}
          required
          placeholder="user@example.com"
        />
        <Input
          label="提供商"
          value={form.provider}
          onChange={(e) => handleChange('provider', e.target.value)}
          required
          placeholder="例如：gmail、outlook"
        />
        <div className="grid grid-cols-2 gap-4">
          <Input
            label="IMAP 主机"
            value={form.imapHost}
            onChange={(e) => handleChange('imapHost', e.target.value)}
            required
            placeholder="imap.gmail.com"
          />
          <Input
            label="IMAP 端口"
            type="number"
            value={form.imapPort}
            onChange={(e) => handleChange('imapPort', parseInt(e.target.value, 10) || 0)}
            required
          />
        </div>
        <div className="grid grid-cols-2 gap-4">
          <Input
            label="SMTP 主机"
            value={form.smtpHost}
            onChange={(e) => handleChange('smtpHost', e.target.value)}
            required
            placeholder="smtp.gmail.com"
          />
          <Input
            label="SMTP 端口"
            type="number"
            value={form.smtpPort}
            onChange={(e) => handleChange('smtpPort', parseInt(e.target.value, 10) || 0)}
            required
          />
        </div>
        <div className="flex items-center gap-6">
          <label className="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
            <input
              type="checkbox"
              checked={form.useOauth}
              onChange={(e) => handleChange('useOauth', e.target.checked)}
              className="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
            />
            使用 OAuth
          </label>
          <label className="flex items-center gap-2 text-sm text-gray-700 dark:text-gray-300">
            <input
              type="checkbox"
              checked={form.isActive}
              onChange={(e) => handleChange('isActive', e.target.checked)}
              className="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500 dark:border-gray-600 dark:bg-gray-700"
            />
            启用
          </label>
        </div>

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
