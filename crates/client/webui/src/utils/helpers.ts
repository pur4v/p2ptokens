import { StoreApi } from 'zustand';
import { IChatStore } from '@/store/createChatStore';
import { IHistoryStore } from '@/store/createHistoryStore';
import { IPreviewStore } from '@/store/createPreviewStore';

export function createNewChat(
  chatStore: StoreApi<IChatStore>,
  historyStore: StoreApi<IHistoryStore>,
  previewStore: StoreApi<IPreviewStore>,
  onNavigate?: (path: string) => void
) {
  const { setActiveThreadId } = chatStore.getState();
  const { setCurrentActiveThread } = historyStore.getState();
  const { setAgentPreviewObject } = previewStore.getState();

  if (onNavigate) onNavigate('');
  setActiveThreadId(null);
  setAgentPreviewObject(null);
  setCurrentActiveThread({ threadId: '' });
}
