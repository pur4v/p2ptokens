import React from 'react';
import { Plus } from 'lucide-react';
import {
  useChatStore,
  useHistoryStore,
  useHistoryStoreState,
  useChatStoreState,
  usePreviewStore,
  useConfig,
  useComponentConfig,
} from '@/provider/ChatProvider';
import { ThreadItem } from '@/components/History/ThreadItem';
import { createNewChat } from '@/utils/helpers';
import { HistoryComponentConfig } from '@/types';

export const HistorySidebar: React.FC = () => {
  const threads = useHistoryStoreState((s) => s.agentThreads);
  const concurrentThreads = useHistoryStoreState((s) => s.concurentThreads);
  const activeThreadId = useChatStoreState((s) => s.activeThreadId);
  const chatStore = useChatStore();
  const historyStore = useHistoryStore();
  const previewStore = usePreviewStore();
  const config = useConfig();
  const componentConfig = useComponentConfig<HistoryComponentConfig>();

  const sidebarWidth = componentConfig.sidebarWidth || '16rem';
  const newChatText = componentConfig.newChatButtonText || config.textConfig?.newChatButtonText || 'New Chat';
  const noConversationsText = componentConfig.noConversationsText || config.textConfig?.noConversationsText || 'No conversations yet';
  const showDeleteButton = componentConfig.showDeleteButton !== false;

  const handleNewChat = () => {
    createNewChat(chatStore, historyStore, previewStore, config.onNavigate);
  };

  const handleThreadClick = (threadId: string) => {
    chatStore.getState().setActiveThreadId(threadId);
    config.onThreadChange?.(threadId);
  };

  const handleDeleteThread = async (threadId: string) => {
    historyStore.getState().deleteThread(threadId);
    chatStore.getState().deleteThreadId(threadId);
    if (activeThreadId === threadId) {
      chatStore.getState().setActiveThreadId(null);
    }
  };

  return (
    <div
      className="fc-flex fc-flex-col fc-h-full fc-border-r"
      style={{ backgroundColor: 'var(--fc-bg)', borderColor: 'var(--fc-assistant-border)', width: sidebarWidth }}
    >
      <div className="fc-p-3 fc-border-b" style={{ borderColor: 'var(--fc-assistant-border)' }}>
        <button
          type="button"
          onClick={handleNewChat}
          className="fc-w-full fc-flex fc-items-center fc-gap-2 fc-px-3 fc-py-2 fc-rounded-lg fc-text-white fc-border-0 fc-cursor-pointer fc-transition-colors fc-text-sm fc-font-medium"
          style={{ backgroundColor: 'var(--fc-primary)' }}
          onMouseEnter={(e) => (e.currentTarget.style.backgroundColor = 'var(--fc-primary-hover)')}
          onMouseLeave={(e) => (e.currentTarget.style.backgroundColor = 'var(--fc-primary)')}
        >
          <Plus size={16} />
          {newChatText}
        </button>
      </div>
      <div className="fc-flex-1 fc-overflow-y-auto fc-p-2 fc-flex fc-flex-col fc-gap-1">
        {threads.map((thread) => (
          <ThreadItem
            key={thread.id}
            thread={thread}
            isActive={activeThreadId === thread.id}
            status={concurrentThreads[thread.id]}
            onClick={handleThreadClick}
            onDelete={showDeleteButton ? handleDeleteThread : undefined}
          />
        ))}
        {threads.length === 0 && (
          <p className="fc-text-center fc-text-sm fc-mt-8" style={{ color: 'var(--fc-text-secondary)' }}>
            {noConversationsText}
          </p>
        )}
      </div>
    </div>
  );
};
