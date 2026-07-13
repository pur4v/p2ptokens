import { StoreApi } from 'zustand';
import { ChatAdapter, ConcurrentThreadStatus } from '@/types';
import { IChatStore } from '@/store/createChatStore';
import { IHistoryStore } from '@/store/createHistoryStore';
import { handleReceivedChunk } from '@/streaming/chunkHandler';
import { AGENT_STREAM_CHUNK_DELIMITER, MAX_CHAT_RETRY_COUNT } from '@/utils/constants';

export async function cancelStreamingTask(
  threadId: string,
  adapter: ChatAdapter
): Promise<void> {
  try {
    await adapter.cancelStream(threadId);
  } catch (error) {
    console.error('Error canceling streaming task:', error);
  }
}

export async function refreshAgentStream(
  threadId: string,
  adapter: ChatAdapter,
  chatStore: StoreApi<IChatStore>,
  historyStore: StoreApi<IHistoryStore>
): Promise<void> {
  const { setThreadStreaming, setThreadAbortController, getChatRetrycount, setChatRetrycount } =
    chatStore.getState();
  const { setConcurrentThreadStatus } = historyStore.getState();

  const controller = new AbortController();
  setThreadAbortController(threadId, controller);
  setThreadStreaming(threadId, true);

  try {
    const stream = await adapter.resumeStream(threadId);
    const reader = stream.getReader();
    const decoder = new TextDecoder();

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      const chunk = decoder.decode(value);
      if (!chunk.trim()) continue;

      const processedChunks = chunk.split(AGENT_STREAM_CHUNK_DELIMITER);
      for (const pchunk of processedChunks) {
        if (!pchunk) continue;
        handleReceivedChunk(pchunk, threadId, chatStore, historyStore);
      }
    }
  } catch (error) {
    throw error instanceof Error ? error : new Error('Unknown error occurred');
  } finally {
    const chatRetrycount = getChatRetrycount();
    if (chatRetrycount >= MAX_CHAT_RETRY_COUNT) {
      setThreadStreaming(threadId, false);
      setThreadAbortController(threadId, null);
      setConcurrentThreadStatus(threadId, ConcurrentThreadStatus.COMPLETED);
    } else {
      try {
        const response = await adapter.fetchThreadMessages(threadId);
        if (response.success) {
          const activeTask = response.data.active_task;
          if (activeTask) {
            setChatRetrycount(chatRetrycount + 1);
            await refreshAgentStream(threadId, adapter, chatStore, historyStore);
          } else {
            setThreadStreaming(threadId, false);
            setThreadAbortController(threadId, null);
            setConcurrentThreadStatus(threadId, ConcurrentThreadStatus.COMPLETED);
            setChatRetrycount(0);
          }
        }
      } catch {
        setThreadStreaming(threadId, false);
        setThreadAbortController(threadId, null);
        setConcurrentThreadStatus(threadId, ConcurrentThreadStatus.COMPLETED);
        setChatRetrycount(0);
      }
    }
  }
}
