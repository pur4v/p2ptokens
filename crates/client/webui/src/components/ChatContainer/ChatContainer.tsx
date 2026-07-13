import React, { useCallback } from 'react';
import clsx from 'clsx';
import { LoaderCircle } from 'lucide-react';
import { useChatStoreState, useChatStore, usePreviewStoreState, usePreviewStore, useHistoryStore, useAdapter, useConfig, useComponentConfig } from '@/provider/ChatProvider';
import { AutoScrollContainer } from '@/components/Shared/AutoScrollContainer';
import { AssistantMessage } from '@/components/Messages/AssistantMessage';
import { UserMessage } from '@/components/Messages/UserMessage';
import { ChatInput } from '@/components/Input/ChatInput';
import { SidePanel } from '@/components/SidePanel/SidePanel';
import { startStreaming } from '@/streaming/streamingService';
import { ChatContainerConfig } from '@/types';

const EMPTY_PROGRESS_TRACKER: any[] = [];

export const ChatContainer: React.FC<{
  loading?: boolean;
  error?: string | null;
}> = ({ loading = false, error = null }) => {
  const chatStore = useChatStore();
  const historyStore = useHistoryStore();
  const previewStore = usePreviewStore();
  const adapter = useAdapter();
  const config = useConfig();
  const componentConfig = useComponentConfig<ChatContainerConfig>();

  const enableComputerTool = componentConfig.enableComputerTool !== false;
  const emptyStateTitle = componentConfig.emptyStateTitle || config.textConfig?.emptyStateTitle || '';
  const emptyStateSubtitle = componentConfig.emptyStateSubtitle || config.textConfig?.emptyStateSubtitle || '';
  const loadingText = config.textConfig?.loadingText || 'Loading chat history...';
  const errorText = config.textConfig?.errorText || 'Failed to load chat';

  const activeThreadId = useChatStoreState((s) => s.activeThreadId);

  const thread = useChatStoreState((s) =>
    activeThreadId ? s.threadsById[activeThreadId] : null
  );

  const isStreaming = useChatStoreState((s) =>
    activeThreadId && s.threadsById[activeThreadId]
      ? s.threadsById[activeThreadId].streaming
      : false
  );

  const progressMessage = useChatStoreState((s) =>
    activeThreadId && s.threadsById[activeThreadId]
      ? s.threadsById[activeThreadId].progressMessage
      : null
  );

  const messages = thread?.messages || [];
  const AgentPreviewObject = usePreviewStoreState((s) => s.AgentPreviewObject);
  const setAgentPreviewObject = usePreviewStoreState((s) => s.setAgentPreviewObject);

  const productMode = useChatStoreState((s) => s.productMode);
  const moduleOutputObject = useChatStoreState((s) => s.moduleOutputObject);

  const progressTracker = useChatStoreState((s) => {
    if (!activeThreadId || !s.threadsById[activeThreadId]) return EMPTY_PROGRESS_TRACKER;
    return s.threadsById[activeThreadId].computerState.progressTracker;
  });

  const handleEditMessage = useCallback(
    async (messageId: string, newContent: string) => {
      if (!activeThreadId || isStreaming) return;

      const store = chatStore.getState();
      const editedMsg = store.editAndResendMessage?.(activeThreadId, messageId, newContent);
      if (!editedMsg) return;

      previewStore.getState().setAgentPreviewObject(null);

      try {
        await startStreaming(
          {
            message: newContent,
            threadId: activeThreadId,
            uploadedImages: editedMsg.images ?? [],
            isNewChat: false,
            productMode,
            selectedKbs: moduleOutputObject.selected_kbs,
            outputResponseFormat: moduleOutputObject.output,
            onThreadIdChange: (newThreadId: string) => {
              config.onThreadChange?.(newThreadId);
            },
          },
          adapter,
          chatStore,
          historyStore
        );
      } catch (err) {
        console.error('Failed to resend edited message:', err);
      }
    },
    [activeThreadId, isStreaming, productMode, moduleOutputObject, adapter, chatStore, historyStore, previewStore, config]
  );

  const isEditEnabled = config.enableMessageEdit && !isStreaming;
  const showSidePanel = config.enableToolPreview !== false && AgentPreviewObject !== null;

  if (loading) {
    return (
      <div className="fc-flex fc-justify-center fc-items-center fc-h-full fc-m-auto" style={{ backgroundColor: 'var(--fc-bg)' }}>
        <div className="fc-text-center fc-flex fc-flex-col fc-items-center">
          <LoaderCircle size={48} className="fc-animate-spin" style={{ color: 'var(--fc-text-secondary)' }} />
          <p className="fc-mt-4" style={{ color: 'var(--fc-text-secondary)' }}>{loadingText}</p>
        </div>
      </div>
    );
  }

  if (error) {
    return (
      <div className="fc-flex fc-justify-center fc-items-center fc-h-full fc-m-auto" style={{ backgroundColor: 'var(--fc-bg)' }}>
        <div className="fc-text-center" style={{ color: 'var(--fc-error)' }}>
          <p>{errorText}</p>
        </div>
      </div>
    );
  }

  if (messages.length === 0) {
    return (
      <div className="fc-flex fc-flex-col fc-items-center fc-h-full fc-w-full fc-max-w-3xl fc-mx-auto fc-justify-center fc-gap-4" style={{ padding: 'var(--fc-spacing-container)' }}>
        {(emptyStateTitle || emptyStateSubtitle) && (
          <div className="fc-text-center fc-flex fc-flex-col fc-gap-2 fc-mb-4">
            {emptyStateTitle && (
              <h2 className="fc-text-2xl fc-font-semibold fc-m-0" style={{ color: 'var(--fc-text)' }}>
                {emptyStateTitle}
              </h2>
            )}
            {emptyStateSubtitle && (
              <p className="fc-text-base fc-m-0" style={{ color: 'var(--fc-text-secondary)' }}>
                {emptyStateSubtitle}
              </p>
            )}
          </div>
        )}
        <ChatInput />
      </div>
    );
  }

  return (
    <div className="fc-flex fc-h-full fc-w-full" style={{ backgroundColor: 'var(--fc-bg)', padding: 'var(--fc-spacing-container)' }}>
      <div className="fc-flex fc-flex-col fc-h-full fc-justify-center fc-w-full fc-flex-1">
        <AutoScrollContainer
          messages={messages}
          activeThreadId={activeThreadId}
          className="fc-flex-1 fc-overflow-y-auto fc-max-w-3xl fc-w-full fc-mx-auto fc-flex fc-flex-col md:fc-p-4"
        >
          {messages.map((message, index) => {
            const isLastMessage = index === messages.length - 1;
            const isCurrentlyStreaming = isLastMessage && isStreaming;

            if (message.role === 'user') {
              return (
                <UserMessage
                  key={`user-${message.id}`}
                  message={message}
                  images={message.images}
                  isEditable={isEditEnabled}
                  onEditSubmit={handleEditMessage}
                />
              );
            } else if (message.role === 'assistant') {
              return (
                <AssistantMessage
                  key={`assistant-${index}`}
                  message={message}
                  isStreaming={isCurrentlyStreaming}
                  progressMessage={progressMessage || ''}
                />
              );
            }
            return null;
          })}
          {messages.length > 0 && <div className="fc-h-20 fc-flex-shrink-0" />}
        </AutoScrollContainer>

        {/* Computer Tool Preview Mini */}
        {enableComputerTool &&
          progressTracker.length > 0 &&
          AgentPreviewObject?.type !== 'computer' &&
          messages.length > 0 && (
            <div className="fc-flex fc-items-center fc-justify-end fc-relative fc-max-w-3xl fc-w-full fc-mx-auto">
              <button
                type="button"
                className="fc-h-16 fc-w-24 fc-rounded-lg fc-overflow-hidden fc-shadow fc-cursor-pointer fc-z-50 fc-absolute fc-top-[-1rem] fc-right-4 fc-flex fc-justify-center fc-items-center fc-border fc-p-0"
                style={{ backgroundColor: 'var(--fc-primary)', borderColor: 'var(--fc-assistant-border)' }}
                onClick={() =>
                  setAgentPreviewObject({
                    title: 'Computer',
                    description: 'Computer',
                    image: '',
                    type: 'computer',
                  })
                }
              >
                <span className="fc-text-xs fc-text-white">Preview</span>
              </button>
            </div>
          )}

        <div className="fc-p-0">
          <ChatInput key={activeThreadId} />
        </div>
      </div>

      {/* Side Panel */}
      {showSidePanel && <SidePanel />}
    </div>
  );
};
