import { StoreApi } from 'zustand';
import { IChatStore } from '@/store/createChatStore';
import { IHistoryStore } from '@/store/createHistoryStore';
import { ProgressTrackerItem } from '@/types';
import { CHAT_INPUT_THINKING_MESSAGE } from '@/utils/constants';

function addInComputer(
  chunk: any,
  threadId: string,
  chatStore: StoreApi<IChatStore>
) {
  const { addProgressTracker } = chatStore.getState();
  try {
    if (chunk.detail.content) {
      chunk.detail.content = JSON.parse(chunk.detail.content);
    }
  } catch (err) {
    console.error({ err, chunk });
  }
  addProgressTracker(threadId, chunk as ProgressTrackerItem);
}

export function handleReceivedChunk(
  chunk: any,
  threadId: string | null,
  chatStore: StoreApi<IChatStore>,
  historyStore: StoreApi<IHistoryStore>
): string | null {
  const {
    addNewThread,
    setThreadTitle,
    setLastMessageId,
    setProgressMessage,
    appendAssistantMessage,
    transformAndAppendAssistantMessage,
  } = chatStore.getState();
  const { appendThread, setCurrentActiveThread } = historyStore.getState();
  const { productMode } = chatStore.getState();

  try {
    if (typeof chunk === 'string') {
      chunk = JSON.parse(chunk);
    }
    if (typeof chunk === 'string') {
      chunk = JSON.parse(chunk);
    }

    switch (chunk.type) {
      case 'thread_created':
        addNewThread(chunk.content);
        return chunk.content;

      case 'thread_title': {
        setThreadTitle(threadId as string, chunk.content);
        appendThread({
          id: threadId as string,
          title: chunk.content,
          product: productMode,
        });
        setCurrentActiveThread({ threadId: threadId as string });
        break;
      }

      case 'assistant_message_created':
        setLastMessageId(threadId as string, chunk.content);
        break;

      case 'streaming':
        setProgressMessage(threadId as string, CHAT_INPUT_THINKING_MESSAGE);
        transformAndAppendAssistantMessage(
          threadId as string,
          chunk,
          (prevChunks, newChunk) => {
            if (prevChunks.length === 0) return [newChunk];
            const lastChunk = prevChunks[prevChunks.length - 1];
            if (lastChunk.type === 'streaming') {
              return [
                ...prevChunks.slice(0, -1),
                {
                  ...lastChunk,
                  content: (lastChunk.content ?? '') + (newChunk.content ?? ''),
                },
              ];
            }
            return [...prevChunks, newChunk];
          }
        );
        break;

      case 'supervisor_streaming':
      case 'security_block':
        setProgressMessage(threadId as string, CHAT_INPUT_THINKING_MESSAGE);
        transformAndAppendAssistantMessage(
          threadId as string,
          chunk,
          (prevChunks, newChunk) => {
            if (prevChunks.length === 0) return [newChunk];
            const lastChunk = prevChunks[prevChunks.length - 1];
            if (lastChunk.type === 'supervisor_streaming') {
              return [
                ...prevChunks.slice(0, -1),
                {
                  ...lastChunk,
                  content: (lastChunk.content ?? '') + (newChunk.content ?? ''),
                },
              ];
            }
            return [...prevChunks, newChunk];
          }
        );
        break;

      case 'toolStart':
        setProgressMessage(threadId as string, CHAT_INPUT_THINKING_MESSAGE);
        transformAndAppendAssistantMessage(
          threadId as string,
          chunk,
          (prevChunks, newChunk) => {
            if (prevChunks.length === 0) return [newChunk];
            return [...prevChunks, newChunk];
          }
        );
        break;

      case 'toolUsed':
        setProgressMessage(threadId as string, CHAT_INPUT_THINKING_MESSAGE);
        transformAndAppendAssistantMessage(
          threadId as string,
          chunk,
          (prevChunks, newChunk) => {
            if (prevChunks.length === 0) return [newChunk];
            return [...prevChunks, newChunk];
          }
        );
        addInComputer(chunk, threadId!, chatStore);
        break;

      case 'consolidated_data':
        setProgressMessage(threadId as string, CHAT_INPUT_THINKING_MESSAGE);
        appendAssistantMessage(threadId as string, chunk);
        break;

      case 'next_questions':
        setProgressMessage(threadId as string, CHAT_INPUT_THINKING_MESSAGE);
        appendAssistantMessage(threadId as string, chunk);
        break;

      default:
        break;
    }
    return null;
  } catch (err) {
    console.log(err, chunk, 'chunkHandler error');
  }
  return threadId;
}
