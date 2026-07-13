import React from 'react';
import { useChatStoreState, usePreviewStoreState } from '@/provider/ChatProvider';
import { ErrorBoundary } from '@/components/Shared/ErrorBoundary';
import { ToolTop } from '@/components/SidePanel/ToolTop';
import { ToolHeader } from '@/components/SidePanel/ToolHeader';
import { ProgressPreview } from '@/components/SidePanel/ProgressPreview';
import { ErrorFallback } from '@/components/SidePanel/tools/ErrorFallback';

const EMPTY_TRACKER: any[] = [];

export const ComputerToolPreview: React.FC = () => {
  const setAgentPreviewObject = usePreviewStoreState((s) => s.setAgentPreviewObject);
  const activeThreadId = useChatStoreState((s) => s.activeThreadId);

  const progressTracker = useChatStoreState((s) => {
    if (!activeThreadId || !s.threadsById[activeThreadId]) return EMPTY_TRACKER;
    return s.threadsById[activeThreadId].computerState.progressTracker;
  });

  const progress = useChatStoreState((s) => {
    if (!activeThreadId || !s.threadsById[activeThreadId]) return 0;
    return s.threadsById[activeThreadId].computerState.activeIndex;
  });

  const currentItem = progressTracker[progress];
  if (!currentItem?.name) return null;

  const toolHeaderMessage = {
    title: currentItem.message?.action ?? '',
    description: currentItem.message?.param ?? '',
  };

  return (
    <div className="fc-h-full fc-w-full fc-flex fc-flex-col fc-py-0 fc-gap-0 fc-px-0 fc-rounded-xl fc-overflow-hidden fc-relative fc-border" style={{ backgroundColor: 'var(--fc-bg)', borderColor: 'var(--fc-border)' }}>
      <ToolTop title="Computer" onClose={() => setAgentPreviewObject(null)} />
      <ErrorBoundary fallback={<ErrorFallback />}>
        <ToolHeader tool={currentItem.name} message={toolHeaderMessage} />
      </ErrorBoundary>
      <ErrorBoundary fallback={<ErrorFallback />}>
        <ProgressPreview />
      </ErrorBoundary>
      <div className="fc-p-2" />
    </div>
  );
};
