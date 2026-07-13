import { StoreApi } from 'zustand';
import { ChatAdapter, ConcurrentThreadStatus, IAssistantChunks, ProductMode, SelectedKB } from '@/types';
import { IChatStore } from '@/store/createChatStore';
import { IHistoryStore } from '@/store/createHistoryStore';
import { handleReceivedChunk } from '@/streaming/chunkHandler';
import { refreshAgentStream } from '@/streaming/resumableStream';
import { AGENT_STREAM_CHUNK_DELIMITER, MAX_CHAT_RETRY_COUNT } from '@/utils/constants';

export interface StreamingOptions {
  message: string;
  threadId: string | null;
  uploadedImages?: any[];
  isNewChat: boolean;
  productMode: ProductMode;
  selectedKbs?: SelectedKB[];
  outputResponseFormat?: string;
  onThreadIdChange?: (newThreadId: string) => void;
}

export const initialAssistantMessage: IAssistantChunks = {
  id: '1',
  content: '',
  detail: { toolUsed: [] },
  name: 'none',
  type: 'none',
};

export async function startStreaming(
  options: StreamingOptions,
  adapter: ChatAdapter,
  chatStore: StoreApi<IChatStore>,
  historyStore: StoreApi<IHistoryStore>
): Promise<void> {
  const {
    setThreadAbortController,
    getLastMessageId,
    deleteThreadId,
    setActiveThreadId,
    setThreadStreaming,
    copyThreadFromTempThreadid,
    appendAssistantMessage,
    setChatRetrycount,
    getChatRetrycount,
  } = chatStore.getState();

  const { setConcurrentThreadStatus } = historyStore.getState();

  let { threadId } = options;
  const lastMessageId = getLastMessageId(threadId);

  try {
    const controller = new AbortController();
    setThreadAbortController(threadId as string, controller);
    setThreadStreaming(threadId as string, true);
    appendAssistantMessage(threadId as string, initialAssistantMessage);

    const stream = await adapter.startStream({
      message: options.message,
      threadId,
      isNewChat: options.isNewChat,
      productMode: options.productMode,
      parentMessageId: lastMessageId,
      images: options.uploadedImages,
      selectedKbs: options.selectedKbs,
      outputResponseFormat: options.outputResponseFormat,
    });

    const reader = stream.getReader();
    const decoder = new TextDecoder();

    if (!options.isNewChat) {
      setConcurrentThreadStatus(threadId as string, ConcurrentThreadStatus.RUNNING);
    }

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const chunk = decoder.decode(value);
      if (!chunk.trim()) continue;

      const processedChunks = chunk.split(AGENT_STREAM_CHUNK_DELIMITER);
      for (const pchunk of processedChunks) {
        if (!pchunk) continue;

        const updatedThreadId = handleReceivedChunk(pchunk, threadId, chatStore, historyStore);
        if (updatedThreadId && updatedThreadId !== threadId) {
          copyThreadFromTempThreadid(updatedThreadId, threadId as string);
          setActiveThreadId(updatedThreadId);
          deleteThreadId(threadId as string);
          setConcurrentThreadStatus(updatedThreadId, ConcurrentThreadStatus.RUNNING);

          if (options.onThreadIdChange) {
            options.onThreadIdChange(updatedThreadId);
          }

          threadId = updatedThreadId;
        }
      }
    }
  } catch (error) {
    throw error instanceof Error ? error : new Error('Unknown error occurred');
  } finally {
    const chatRetrycount = getChatRetrycount();
    if (chatRetrycount >= MAX_CHAT_RETRY_COUNT) {
      setThreadStreaming(threadId as string, false);
      setThreadAbortController(threadId as string, null);
      setConcurrentThreadStatus(threadId as string, ConcurrentThreadStatus.COMPLETED);
    } else {
      try {
        const response = await adapter.fetchThreadMessages(threadId as string);
        if (response.success) {
          const activeTask = response.data.active_task;
          if (activeTask) {
            setChatRetrycount(chatRetrycount + 1);
            await refreshAgentStream(threadId as string, adapter, chatStore, historyStore);
          } else {
            setThreadStreaming(threadId as string, false);
            setThreadAbortController(threadId as string, null);
            setConcurrentThreadStatus(threadId as string, ConcurrentThreadStatus.COMPLETED);
            setChatRetrycount(0);
          }
        }
      } catch {
        setThreadStreaming(threadId as string, false);
        setThreadAbortController(threadId as string, null);
        setConcurrentThreadStatus(threadId as string, ConcurrentThreadStatus.COMPLETED);
        setChatRetrycount(0);
      }
    }
  }
}
