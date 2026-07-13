import { createStore, StoreApi } from 'zustand';
import { ConcurrentThreadStatus, IThread } from '@/types';

export interface IHistoryStore {
  agentThreads: IThread[];
  setAgentThreads: (threads: IThread[] | ((prev: IThread[]) => IThread[])) => void;
  appendThread: (thread: IThread) => void;
  updateThread: (threadId: string, updatedThread: Partial<IThread>) => void;
  deleteThread: (threadId: string) => void;

  concurentThreads: Record<string, ConcurrentThreadStatus>;
  setConcurrentThreadStatus: (threadId: string, status: ConcurrentThreadStatus) => void;
  deleteConcurrentThreadStatus: (threadId: string) => void;

  currentActiveThread: {
    threadId?: string;
    lastMessageId?: string;
  };
  setCurrentActiveThread: (obj: { threadId?: string; lastMessageId?: string }) => void;

  historyLoaded: boolean;
  setHistoryLoaded: (value: boolean) => void;
}

export function createHistoryStore(): StoreApi<IHistoryStore> {
  return createStore<IHistoryStore>((set) => ({
    concurentThreads: {},

    setConcurrentThreadStatus: (threadId, status) =>
      set((state) => ({
        concurentThreads: { ...state.concurentThreads, [threadId]: status },
      })),

    deleteConcurrentThreadStatus: (threadId) =>
      set((state) => {
        const { [threadId]: _, ...rest } = state.concurentThreads;
        return { concurentThreads: rest };
      }),

    currentActiveThread: {},
    setCurrentActiveThread: (obj) =>
      set((state) => ({
        currentActiveThread: { ...state.currentActiveThread, ...obj },
      })),

    historyLoaded: false,
    setHistoryLoaded: (value) => set({ historyLoaded: value }),

    agentThreads: [],
    setAgentThreads: (threads) => {
      set((state) => ({
        agentThreads: typeof threads === 'function' ? threads(state.agentThreads) : threads,
      }));
      if (typeof threads !== 'function') {
        threads?.forEach((thread) => {
          if (thread.is_running) {
            set((state) => ({
              concurentThreads: {
                ...state.concurentThreads,
                [thread.id]: ConcurrentThreadStatus.RUNNING,
              },
            }));
          }
        });
      }
    },

    appendThread: (thread) =>
      set((state) => ({ agentThreads: [thread, ...state.agentThreads] })),

    updateThread: (threadId, updatedThread) =>
      set((state) => ({
        agentThreads: state.agentThreads.map((t) =>
          t.id === threadId ? { ...t, ...updatedThread } : t
        ),
      })),

    deleteThread: (threadId) =>
      set((state) => ({
        agentThreads: state.agentThreads.filter((t) => t.id !== threadId),
      })),
  }));
}
