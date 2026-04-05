import { useState, useCallback } from 'react';
import { Loader2, Sparkles } from 'lucide-react';
import Modal from './ui/Modal';
import Button from './ui/Button';
import Badge from './ui/Badge';
import Card from './ui/Card';
import type { WorkItem, AnalysisType } from '../types/domain';
import * as api from '../services/api';

interface AiAnalysisPanelProps {
  workItem: WorkItem;
  isOpen: boolean;
  onClose: () => void;
}

interface AnalysisResult {
  type: AnalysisType;
  label: string;
  content: string;
}

const analysisOptions: { type: AnalysisType; label: string }[] = [
  { type: 'summary', label: '摘要' },
  { type: 'actionExtraction', label: '行动提取' },
  { type: 'classification', label: '分类' },
  { type: 'prioritySuggestion', label: '优先级建议' },
];

export default function AiAnalysisPanel({
  workItem,
  isOpen,
  onClose,
}: AiAnalysisPanelProps) {
  const [results, setResults] = useState<AnalysisResult[]>([]);
  const [loadingType, setLoadingType] = useState<AnalysisType | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleAnalyze = useCallback(
    async (type: AnalysisType, label: string) => {
      setLoadingType(type);
      setError(null);
      try {
        const content = await api.analyzeWorkItem(workItem.id, type);
        setResults((prev) => {
          const filtered = prev.filter((r) => r.type !== type);
          return [...filtered, { type, label, content }];
        });
      } catch (err: unknown) {
        const msg = err instanceof Error ? err.message : '分析失败，请重试';
        setError(msg);
      } finally {
        setLoadingType(null);
      }
    },
    [workItem.id],
  );

  const handleClose = useCallback(() => {
    setResults([]);
    setError(null);
    setLoadingType(null);
    onClose();
  }, [onClose]);

  return (
    <Modal isOpen={isOpen} onClose={handleClose} title="AI 分析">
      <div className="space-y-4">
        {/* Work item title */}
        <div className="flex items-center gap-2">
          <Sparkles className="h-4 w-4 text-purple-500" />
          <span className="text-sm font-medium text-gray-900 dark:text-white">
            {workItem.title}
          </span>
        </div>

        {/* Analysis type buttons */}
        <div className="flex flex-wrap gap-2">
          {analysisOptions.map(({ type, label }) => (
            <Button
              key={type}
              variant="secondary"
              size="sm"
              disabled={loadingType !== null}
              onClick={() => handleAnalyze(type, label)}
            >
              {loadingType === type && (
                <Loader2 className="h-3.5 w-3.5 animate-spin" />
              )}
              {label}
            </Button>
          ))}
        </div>

        {/* Error message */}
        {error && (
          <p className="text-sm text-red-600 dark:text-red-400">{error}</p>
        )}

        {/* Loading indicator for current analysis */}
        {loadingType && (
          <div className="flex items-center gap-2 py-3">
            <Loader2 className="h-4 w-4 animate-spin text-purple-500" />
            <span className="text-sm text-gray-500 dark:text-gray-400">
              正在分析...
            </span>
          </div>
        )}

        {/* Stacked results */}
        {results.length > 0 && (
          <div className="max-h-64 space-y-3 overflow-y-auto">
            {results.map((result) => (
              <Card key={result.type}>
                <div className="space-y-2">
                  <Badge variant="info">{result.label}</Badge>
                  <p className="whitespace-pre-wrap text-sm text-gray-700 dark:text-gray-300">
                    {result.content}
                  </p>
                </div>
              </Card>
            ))}
          </div>
        )}
      </div>
    </Modal>
  );
}
