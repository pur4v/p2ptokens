import React from 'react';
import { Sparkles } from 'lucide-react';
import { useChatStore, useAdapter, useConfig } from '@/provider/ChatProvider';
import { IUserMessage } from '@/types';
import { CHAT_INPUT_THINKING_MESSAGE } from '@/utils/constants';
import { startStreaming } from '@/streaming/streamingService';
import { useHistoryStore } from '@/provider/ChatProvider';

interface FollowUpQuestionsProps {
  nextQuestions: string[];
}

export const FollowUpQuestions: React.FC<FollowUpQuestionsProps> = ({ nextQuestions }) => {
  const chatStore = useChatStore();
  const historyStore = useHistoryStore();
  const adapter = useAdapter();
  const config = useConfig();
  const followUpTitle = config.textConfig?.followUpTitle || 'Suggested Follow-ups:';

  const handleSubmit = async (query: string) => {
    const store = chatStore.getState();
    const {
      activeThreadId,
      productMode,
      moduleOutputObject,
      addNewThread,
      appendUserMessage,
      setActiveThreadId,
      getThreadById,
      getThreadStreaming,
      setProgressMessage,
    } = store;

    const isStreaming = activeThreadId ? getThreadStreaming(activeThreadId) : false;
    if (isStreaming) return;

    try {
      let currentThreadId = activeThreadId;
      let isNewChat = false;

      if (!activeThreadId || !getThreadById(activeThreadId)) {
        currentThreadId = `temp_${Date.now()}`;
        addNewThread(currentThreadId);
        setActiveThreadId(currentThreadId);
        isNewChat = true;
      }

      const userMessage: IUserMessage = {
        role: 'user',
        id: Date.now().toString(),
        content: query.trim(),
        images: [],
        selected_kbs: moduleOutputObject.selected_kbs,
        outputResponseFormat: moduleOutputObject.output,
      };

      setProgressMessage(currentThreadId as string, CHAT_INPUT_THINKING_MESSAGE);
      appendUserMessage(currentThreadId as string, userMessage);

      await startStreaming(
        {
          message: query.trim(),
          threadId: currentThreadId as string,
          uploadedImages: [],
          isNewChat,
          productMode,
          selectedKbs: moduleOutputObject.selected_kbs,
          outputResponseFormat: moduleOutputObject.output,
        },
        adapter,
        chatStore,
        historyStore
      );
    } catch (error) {
      console.error('Failed to send follow-up:', error);
    }
  };

  return (
    <div className="fc-flex fc-flex-col fc-border-t fc-pt-4 fc-mx-4" style={{ borderColor: 'var(--fc-assistant-border)' }}>
      <div className="fc-flex fc-items-center fc-gap-2">
        <div className="fc-p-2 fc-rounded-full" style={{ backgroundColor: 'color-mix(in srgb, var(--fc-primary) 10%, transparent)' }}>
          <Sparkles size={16} className="fc-shrink-0" style={{ color: 'var(--fc-primary)' }} />
        </div>
        <span className="fc-font-medium fc-text-sm" style={{ color: 'var(--fc-text)' }}>{followUpTitle}</span>
      </div>
      <div className="fc-flex fc-flex-col fc-py-3 md:fc-p-3 fc-gap-2">
        {nextQuestions.map((question, i) => (
          <button
            key={i}
            type="button"
            onClick={() => handleSubmit(question)}
            className="fc-cursor-pointer fc-w-full fc-border fc-rounded-lg fc-px-2 fc-py-1.5 md:fc-px-3 md:fc-py-2 fc-bg-transparent fc-text-left fc-flex fc-justify-between fc-items-center fc-gap-4 fc-transition-colors"
            style={{ borderColor: 'var(--fc-assistant-border)' }}
            onMouseEnter={(e) => (e.currentTarget.style.borderColor = 'var(--fc-primary)')}
            onMouseLeave={(e) => (e.currentTarget.style.borderColor = 'var(--fc-assistant-border)')}
          >
            <span className="fc-text-[10px] md:fc-text-xs" style={{ color: 'var(--fc-text)' }}>
              {question}
            </span>
          </button>
        ))}
      </div>
    </div>
  );
};
