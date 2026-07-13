import React, { useCallback, useEffect, useRef, useState } from 'react';
import clsx from 'clsx';
import { CircleStop, LoaderCircle, Send } from 'lucide-react';
import TextareaAutosize from 'react-textarea-autosize';
import {
  useChatStore,
  useChatStoreState,
  useAdapter,
  useConfig,
  useHistoryStore,
  useComponentConfig,
} from '@/provider/ChatProvider';
import { ChatInputComponentConfig, IUserMessage } from '@/types';
import { CHAT_INPUT_THINKING_MESSAGE } from '@/utils/constants';
import { startStreaming } from '@/streaming/streamingService';
import { cancelStreamingTask } from '@/streaming/resumableStream';
import { useFileUpload } from '@/hooks/useFileUpload';
import { FileUploadButton } from '@/components/Input/FileUploadButton';
import { FilePreview } from '@/components/Input/FilePreview';

interface ChatInputProps {
  initialQuery?: string;
}

export const ChatInput: React.FC<ChatInputProps> = ({ initialQuery }) => {
  const [query, setQuery] = useState(initialQuery || '');
  const [isStoppingStream, setIsStoppingStream] = useState(false);
  const textareaRef = useRef<HTMLTextAreaElement>(null);
  const chatStore = useChatStore();
  const historyStore = useHistoryStore();
  const adapter = useAdapter();
  const config = useConfig();
  const componentConfig = useComponentConfig<ChatInputComponentConfig>();

  const placeholder = componentConfig.placeholder || config.textConfig?.inputPlaceholder || 'How can I help you today?';
  const maxRows = componentConfig.maxRows || 7;
  const minRows = componentConfig.minRows || 2;
  const maxFileUploads = componentConfig.maxFileUploads || 5;

  const activeThreadId = useChatStoreState((s) => s.activeThreadId);
  const productMode = useChatStoreState((s) => s.productMode);
  const moduleOutputObject = useChatStoreState((s) => s.moduleOutputObject);

  const isCurrentlyStreaming = useChatStoreState((s) =>
    activeThreadId && s.threadsById[activeThreadId]
      ? s.threadsById[activeThreadId].streaming
      : false
  );

  const threadAbortController = useChatStoreState((s) =>
    activeThreadId && s.threadsById[activeThreadId]
      ? s.threadsById[activeThreadId].abortController
      : null
  );

  const fileUpload = useFileUpload({ maxFiles: maxFileUploads });

  useEffect(() => {
    if (initialQuery) setQuery(initialQuery);
  }, [initialQuery]);

  const handleSubmit = useCallback(
    async (overrideQuery?: string) => {
      const messageQuery = overrideQuery ?? query;
      if (!messageQuery.trim()) return;

      const store = chatStore.getState();
      const {
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

        const uploadedImages = fileUpload.getUploadedImages();

        const userMessage: IUserMessage = {
          role: 'user',
          id: Date.now().toString(),
          content: messageQuery.trim(),
          images: uploadedImages.length > 0 ? uploadedImages : undefined,
          selected_kbs: moduleOutputObject.selected_kbs,
          outputResponseFormat: moduleOutputObject.output,
        };

        setProgressMessage(currentThreadId as string, CHAT_INPUT_THINKING_MESSAGE);
        appendUserMessage(currentThreadId as string, userMessage);
        setQuery('');
        fileUpload.clearAll();

        await startStreaming(
          {
            message: messageQuery.trim(),
            threadId: currentThreadId as string,
            uploadedImages,
            isNewChat,
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
      } catch (error) {
        console.error('Failed to send message:', error);
      }
    },
    [query, activeThreadId, productMode, moduleOutputObject, adapter, chatStore, historyStore, config, fileUpload]
  );

  const onStop = useCallback(async () => {
    if (threadAbortController && !isStoppingStream) {
      try {
        setIsStoppingStream(true);
        chatStore.getState().setProgressMessage(activeThreadId as string, CHAT_INPUT_THINKING_MESSAGE);
        await cancelStreamingTask(activeThreadId as string, adapter);
      } catch (err) {
        console.error('Failed to cancel streaming task:', err);
      } finally {
        setIsStoppingStream(false);
      }
    }
  }, [threadAbortController, activeThreadId, isStoppingStream, adapter, chatStore]);

  const onKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      if (isCurrentlyStreaming) {
        onStop();
      } else {
        handleSubmit();
      }
    }
  };

  const canSubmit = (query.trim() || fileUpload.files.length > 0) && !fileUpload.isUploading;

  return (
    <div className={`${activeThreadId ? 'fc-pt-2 md:fc-pt-3 md:fc-border-t fc-flex fc-w-full' : 'fc-flex fc-w-full'}`} style={activeThreadId ? { borderColor: 'var(--fc-assistant-border)' } : undefined}>
      <div className="fc-flex fc-flex-col fc-gap-2 fc-w-full fc-max-w-3xl fc-mx-auto">
        <div
          className={clsx(
            'fc-relative fc-flex fc-flex-col fc-border fc-p-2 md:fc-px-3 md:fc-py-2 fc-gap-3',
            !activeThreadId && 'fc-shadow-lg'
          )}
          style={{
            borderRadius: 'var(--fc-radius-input)',
            borderColor: 'var(--fc-assistant-border)',
            backgroundColor: 'var(--fc-bg)',
          }}
        >
          {/* File previews */}
          {config.enableFileUpload && fileUpload.files.length > 0 && (
            <FilePreview
              files={fileUpload.files}
              onRemove={fileUpload.removeFile}
              onRetry={fileUpload.retryFile}
            />
          )}

          <TextareaAutosize
            ref={textareaRef}
            placeholder={placeholder}
            value={query}
            onChange={(e) => setQuery(e.target.value)}
            onKeyDown={onKeyDown}
            onPaste={config.enableFileUpload ? fileUpload.handlePaste : undefined}
            maxRows={maxRows}
            minRows={minRows}
            style={{ height: 50, color: 'var(--fc-text)' }}
            className="fc-outline-none fc-resize-none fc-bg-transparent fc-text-sm md:fc-text-base fc-w-full fc-pt-1 fc-pl-3 fc-border-0 fc-font-sans"
          />
          <div className="fc-flex fc-justify-end fc-gap-2 fc-items-center fc-text-xs">
            {config.enableFileUpload && (
              <div className="fc-mr-auto">
                <FileUploadButton
                  onFilesSelected={fileUpload.uploadFiles}
                  disabled={fileUpload.files.length >= maxFileUploads}
                />
              </div>
            )}
            <button
              type="button"
              disabled={
                (!canSubmit && !isCurrentlyStreaming) || isStoppingStream
              }
              onClick={isCurrentlyStreaming ? onStop : () => handleSubmit()}
              className="fc-w-8 fc-h-8 md:fc-w-9 md:fc-h-9 fc-transition-opacity hover:fc-opacity-90 fc-rounded-xl disabled:fc-opacity-50 disabled:fc-cursor-not-allowed fc-flex fc-items-center fc-justify-center fc-border-0 fc-cursor-pointer"
              style={{ backgroundColor: 'var(--fc-primary)' }}
            >
              {isStoppingStream ? (
                <LoaderCircle strokeWidth={2} size={16} color="white" />
              ) : isCurrentlyStreaming ? (
                <CircleStop strokeWidth={2} size={16} color="white" />
              ) : (
                <Send strokeWidth={2} size={16} color="white" />
              )}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};

ChatInput.displayName = 'ChatInput';
