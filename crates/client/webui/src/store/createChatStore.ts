import { createStore, StoreApi } from 'zustand';
import {
  IAssistantChunks,
  IAssistantMessage,
  IThreadState,
  IUserMessage,
  ModuleOutputObject,
  ProductMode,
  ProgressTrackerItem,
  SelectedKB,
} from '@/types';
import { CHAT_INPUT_THINKING_MESSAGE, PRODUCT_DEVAS } from '@/utils/constants';

export interface IChatStore {
  threadsById: Record<string, IThreadState>;
  activeThreadId: string | null;
  productMode: ProductMode;
  dislikeReasons: any[];
  responseStepsList: any[];
  moduleOutputObject: ModuleOutputObject;

  // Thread ID
  getActiveThreadId: () => string | null;
  setActiveThreadId: (threadId: string | null) => void;

  // Product mode
  setProductMode: (mode: ProductMode) => void;
  getProductMode: () => ProductMode;

  // Thread CRUD
  getAllThreads: () => Record<string, IThreadState>;
  getThreadById: (threadId: string | null) => IThreadState | null;
  addNewThread: (threadId: string) => void;
  deleteThreadId: (threadId: string) => void;
  copyThreadFromTempThreadid: (threadId: string, tempThreadId: string) => void;

  // Thread metadata
  getLastMessageId: (threadId: string | null) => string | null;
  setLastMessageId: (threadId: string, lastMessageId: string | null) => void;
  setThreadTitle: (threadId: string, title: string) => void;
  setProgressMessage: (threadId: string, progressMessage: string) => void;
  getProgressMessage: (threadId: string | null) => string | null;

  // Streaming control
  setThreadAbortController: (threadId: string, controller: AbortController | null) => void;
  getThreadAbortController: (threadId: string | null) => AbortController | null;
  getThreadStreaming: (threadId: string | null) => boolean;
  setThreadStreaming: (threadId: string, streaming: boolean) => void;

  // Messages
  appendUserMessage: (threadId: string, message: IUserMessage) => void;
  appendAssistantMessage: (threadId: string, message: IAssistantChunks) => void;
  appendAssistantMessageWithId: (
    threadId: string,
    messageId: string,
    chunks: IAssistantChunks[],
    reaction?: IAssistantMessage['reaction']
  ) => void;
  transformAndAppendAssistantMessage: (
    threadId: string,
    chunk: IAssistantChunks,
    transform: (prevChunks: IAssistantChunks[], newChunk: IAssistantChunks) => IAssistantChunks[]
  ) => void;

  // Computer state
  setActiveIndex: (threadId: string, index: number) => void;
  getActiveIndex: (threadId: string) => number;
  incrementActiveIndex: (threadId: string) => void;
  decrementActiveIndex: (threadId: string) => void;
  jumpToLive: (threadId: string) => void;
  jumpToCallId: (threadId: string, callId: string) => void;
  setShouldJumpToLive: (threadId: string, shouldJump: boolean) => void;
  getShouldJumpToLive: (threadId: string) => boolean;

  // Progress tracker
  addProgressTracker: (threadId: string, progressTracker: ProgressTrackerItem) => void;
  clearProgressTracker: (threadId: string) => void;
  getProgressTracker: (threadId: string) => ProgressTrackerItem[];
  addProgressIdMap: (threadId: string, progressId: string, progress: number) => void;
  clearProgressIdMap: (threadId: string) => void;
  getProgressIdMap: (threadId: string) => Record<string, number>;

  // Reactions
  setAssistantMessageReaction: (
    threadId: string,
    messageId: string,
    reaction: NonNullable<IAssistantMessage['reaction']>
  ) => void;
  setDislikeReasons: (reasons: any[]) => void;
  getDislikeReasons: () => any[];

  // Response steps
  addResponseStep: (threadId: string, responseStep: any) => void;
  removeResponseStep: (threadId: string, responseStep: any) => void;
  clearResponseStepsList: (threadId: string) => void;
  updateResponseStep: (threadId: string, responseStep: any) => void;
  getResponseStepsList: (threadId: string) => any[];
  getResponseStepById: (threadId: string, stepId: string) => any | null;

  // Module output
  setModuleOutputObject: (moduleOutputObject: ModuleOutputObject) => void;
  resetModuleOutputObject: () => void;

  // Retry
  chatRetrycount: number;
  setChatRetrycount: (retrycount: number) => void;
  getChatRetrycount: () => number;

  // Edit message
  truncateMessagesAfter: (threadId: string, messageIndex: number) => void;
  editAndResendMessage: (
    threadId: string,
    messageId: string,
    newContent: string
  ) => IUserMessage | null;
}

const createDefaultThreadState = (): IThreadState => ({
  messages: [],
  lastMessageId: null,
  title: '',
  abortController: null,
  progressMessage: CHAT_INPUT_THINKING_MESSAGE,
  streaming: false,
  computerState: {
    activeIndex: 0,
    shouldJumpToLive: false,
    progressTracker: [],
    progressIdMap: {},
  },
  responseStepsList: [],
  moduleOutputObject: {
    output: '',
    selected_kbs: [],
  },
});

const DEFAULT_THREAD_STATE: IThreadState = createDefaultThreadState();

export function createChatStore(): StoreApi<IChatStore> {
  return createStore<IChatStore>((set, get) => ({
    threadsById: {},
    activeThreadId: null,
    productMode: PRODUCT_DEVAS as ProductMode,
    dislikeReasons: [],
    responseStepsList: [],
    moduleOutputObject: { output: '', selected_kbs: [] },

    setModuleOutputObject: (moduleOutputObject) => set({ moduleOutputObject }),
    resetModuleOutputObject: () => set({ moduleOutputObject: { output: '', selected_kbs: [] } }),

    getActiveThreadId: () => get().activeThreadId,
    setActiveThreadId: (threadId) => set({ activeThreadId: threadId }),
    setProductMode: (mode) => set({ productMode: mode }),
    getProductMode: () => get().productMode,

    getAllThreads: () => get().threadsById,
    getThreadById: (threadId) => {
      if (!threadId) return null;
      return get().threadsById[threadId] || null;
    },

    addNewThread: (threadId) => {
      set((state) => {
        if (state.threadsById[threadId]) return state;
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: createDefaultThreadState(),
          },
        };
      });
    },

    deleteThreadId: (threadId) => {
      set((state) => {
        const { [threadId]: _, ...rest } = state.threadsById;
        return { threadsById: rest };
      });
    },

    copyThreadFromTempThreadid: (threadId, tempThreadId) => {
      set((state) => {
        const threadState = state.threadsById[tempThreadId] || createDefaultThreadState();
        const { [tempThreadId]: _, ...rest } = state.threadsById;
        return {
          threadsById: { ...rest, [threadId]: { ...threadState } },
        };
      });
    },

    getLastMessageId: (threadId) => {
      if (!threadId) return null;
      return get().threadsById[threadId]?.lastMessageId ?? null;
    },

    setLastMessageId: (threadId, lastMessageId) => {
      set((state) => {
        const prev = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: { ...state.threadsById, [threadId]: { ...prev, lastMessageId } },
        };
      });
    },

    setThreadTitle: (threadId, title) => {
      set((state) => {
        const prev = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: { ...state.threadsById, [threadId]: { ...prev, title } },
        };
      });
    },

    setProgressMessage: (threadId, progressMessage) => {
      set((state) => {
        const prev = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: { ...state.threadsById, [threadId]: { ...prev, progressMessage } },
        };
      });
    },

    getProgressMessage: (threadId) => {
      if (!threadId) return null;
      return get().threadsById[threadId]?.progressMessage ?? null;
    },

    setThreadAbortController: (threadId, controller) => {
      set((state) => {
        if (!threadId) return state;
        const prev = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: { ...prev, abortController: controller },
          },
        };
      });
    },

    getThreadAbortController: (threadId) => {
      if (!threadId) return null;
      return get().threadsById[threadId]?.abortController ?? null;
    },

    getThreadStreaming: (threadId) => {
      if (!threadId) return false;
      return get().threadsById[threadId]?.streaming ?? false;
    },

    setThreadStreaming: (threadId, streaming) => {
      set((state) => {
        const prev = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: { ...state.threadsById, [threadId]: { ...prev, streaming } },
        };
      });
    },

    appendUserMessage: (threadId, message) => {
      set((state) => {
        const threadState = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: { ...threadState, messages: [...threadState.messages, message] },
          },
        };
      });
    },

    appendAssistantMessage: (threadId, chunk) => {
      set((state) => {
        const threadState = state.threadsById[threadId] || createDefaultThreadState();
        const messages = [...threadState.messages];
        const lastMessage = messages[messages.length - 1];

        if (lastMessage && lastMessage.role === 'assistant') {
          const updated: IAssistantMessage = {
            ...lastMessage,
            assistantChunks: [...lastMessage.assistantChunks, chunk],
          };
          messages[messages.length - 1] = updated;
        } else {
          messages.push({
            role: 'assistant',
            id: Date.now().toString(),
            assistantChunks: [chunk],
          });
        }

        return {
          threadsById: { ...state.threadsById, [threadId]: { ...threadState, messages } },
        };
      });
    },

    transformAndAppendAssistantMessage: (threadId, chunk, transform) => {
      set((state) => {
        const threadState = state.threadsById[threadId] || createDefaultThreadState();
        const messages = [...threadState.messages];
        const lastMessage = messages[messages.length - 1];

        if (lastMessage && lastMessage.role === 'assistant') {
          const last = lastMessage as IAssistantMessage;
          const newChunks = transform(last.assistantChunks, chunk);
          messages[messages.length - 1] = { ...last, assistantChunks: newChunks };
        } else {
          messages.push({
            role: 'assistant',
            id: Date.now().toString(),
            assistantChunks: [chunk],
          });
        }

        return {
          threadsById: { ...state.threadsById, [threadId]: { ...threadState, messages } },
        };
      });
    },

    appendAssistantMessageWithId: (threadId, messageId, chunks, reaction) => {
      set((state) => {
        const threadState = state.threadsById[threadId] || createDefaultThreadState();
        const assistantMessage: IAssistantMessage = {
          role: 'assistant',
          id: messageId,
          assistantChunks: chunks,
          reaction,
        };
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: { ...threadState, messages: [...threadState.messages, assistantMessage] },
          },
        };
      });
    },

    // ---- Computer State ----

    setActiveIndex: (threadId, index) => {
      set((state) => {
        const ts = state.threadsById[threadId] || DEFAULT_THREAD_STATE;
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              computerState: { ...ts.computerState, activeIndex: index },
            },
          },
        };
      });
    },

    getActiveIndex: (threadId) =>
      (get().threadsById[threadId] || DEFAULT_THREAD_STATE).computerState.activeIndex,

    incrementActiveIndex: (threadId) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        const curr = ts.computerState.activeIndex;
        const max = ts.computerState.progressTracker.length - 1;
        if (curr >= max) return state;
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              computerState: { ...ts.computerState, activeIndex: Math.min(max, curr + 1) },
            },
          },
        };
      });
    },

    decrementActiveIndex: (threadId) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        const curr = ts.computerState.activeIndex;
        if (curr <= 0) return state;
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              computerState: {
                ...ts.computerState,
                activeIndex: Math.max(0, curr - 1),
                shouldJumpToLive: false,
              },
            },
          },
        };
      });
    },

    jumpToLive: (threadId) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        const max = ts.computerState.progressTracker.length - 1;
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              computerState: {
                ...ts.computerState,
                activeIndex: Math.max(0, max),
                shouldJumpToLive: true,
              },
            },
          },
        };
      });
    },

    jumpToCallId: (threadId, callId) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        const targetIndex = ts.computerState.progressIdMap[callId];
        if (targetIndex === undefined) return state;
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              computerState: {
                ...ts.computerState,
                activeIndex: targetIndex,
                shouldJumpToLive: false,
              },
            },
          },
        };
      });
    },

    setShouldJumpToLive: (threadId, shouldJump) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              computerState: { ...ts.computerState, shouldJumpToLive: shouldJump },
            },
          },
        };
      });
    },

    getShouldJumpToLive: (threadId) =>
      (get().threadsById[threadId] || DEFAULT_THREAD_STATE).computerState.shouldJumpToLive,

    addProgressTracker: (threadId, progressTracker) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        const current = ts.computerState.progressTracker;
        const newIdMap = {
          ...ts.computerState.progressIdMap,
          [progressTracker.call_id]: current.length,
        };
        const updated = {
          ...state,
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              computerState: {
                ...ts.computerState,
                progressTracker: [...current, progressTracker],
                progressIdMap: newIdMap,
              },
            },
          },
        };
        if (ts.computerState.shouldJumpToLive) {
          updated.threadsById[threadId].computerState.activeIndex = current.length;
        }
        return updated;
      });
    },

    clearProgressTracker: (threadId) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              computerState: {
                ...ts.computerState,
                progressTracker: [],
                progressIdMap: {},
                activeIndex: 0,
                shouldJumpToLive: true,
              },
            },
          },
        };
      });
    },

    getProgressTracker: (threadId) =>
      (get().threadsById[threadId] || DEFAULT_THREAD_STATE).computerState.progressTracker,

    addProgressIdMap: (threadId, progressId, index) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              computerState: {
                ...ts.computerState,
                progressIdMap: { ...ts.computerState.progressIdMap, [progressId]: index },
              },
            },
          },
        };
      });
    },

    clearProgressIdMap: (threadId) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              computerState: { ...ts.computerState, progressIdMap: {} },
            },
          },
        };
      });
    },

    getProgressIdMap: (threadId) =>
      (get().threadsById[threadId] || DEFAULT_THREAD_STATE).computerState.progressIdMap,

    // ---- Reactions ----

    setAssistantMessageReaction: (threadId, messageId, reaction) => {
      set((state) => {
        const ts = state.threadsById[threadId];
        if (!ts) return state;
        const messages = [...ts.messages];
        const idx = messages.findIndex((m) => m.id === messageId && m.role === 'assistant');
        if (idx !== -1) {
          messages[idx] = { ...(messages[idx] as IAssistantMessage), reaction };
        }
        return {
          threadsById: { ...state.threadsById, [threadId]: { ...ts, messages } },
        };
      });
    },

    setDislikeReasons: (reasons) => set({ dislikeReasons: reasons }),
    getDislikeReasons: () => get().dislikeReasons,

    // ---- Response Steps ----

    addResponseStep: (threadId, responseStep) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              responseStepsList: [...ts.responseStepsList, responseStep],
            },
          },
        };
      });
    },

    removeResponseStep: (threadId, responseStep) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              responseStepsList: ts.responseStepsList.filter((s) => s !== responseStep),
            },
          },
        };
      });
    },

    clearResponseStepsList: (threadId) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: { ...ts, responseStepsList: [] },
          },
        };
      });
    },

    updateResponseStep: (threadId, responseStep) => {
      set((state) => {
        const ts = state.threadsById[threadId] || createDefaultThreadState();
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              responseStepsList: ts.responseStepsList.map((s) =>
                s.id === responseStep.id ? responseStep : s
              ),
            },
          },
        };
      });
    },

    getResponseStepsList: (threadId) =>
      get().threadsById[threadId]?.responseStepsList || [],

    getResponseStepById: (threadId, stepId) =>
      get().threadsById[threadId]?.responseStepsList.find((s: any) => s.id === stepId) || null,

    // ---- Retry ----

    chatRetrycount: 0,
    setChatRetrycount: (retrycount) => set({ chatRetrycount: retrycount }),
    getChatRetrycount: () => get().chatRetrycount,

    // ---- Edit Message ----

    truncateMessagesAfter: (threadId, messageIndex) => {
      set((state) => {
        const ts = state.threadsById[threadId];
        if (!ts) return state;
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...ts,
              messages: ts.messages.slice(0, messageIndex + 1),
              responseStepsList: [],
              computerState: {
                ...ts.computerState,
                progressTracker: [],
                progressIdMap: {},
                activeIndex: 0,
                shouldJumpToLive: true,
              },
            },
          },
        };
      });
    },

    editAndResendMessage: (threadId, messageId, newContent) => {
      const ts = get().threadsById[threadId];
      if (!ts) return null;

      const messageIndex = ts.messages.findIndex(
        (m) => m.id === messageId && m.role === 'user'
      );
      if (messageIndex === -1) return null;

      const originalMessage = ts.messages[messageIndex] as IUserMessage;
      const updatedMessage: IUserMessage = {
        ...originalMessage,
        content: newContent,
      };

      set((state) => {
        const threadState = state.threadsById[threadId];
        if (!threadState) return state;
        const truncatedMessages = threadState.messages.slice(0, messageIndex);
        return {
          threadsById: {
            ...state.threadsById,
            [threadId]: {
              ...threadState,
              messages: [...truncatedMessages, updatedMessage],
              lastMessageId: messageIndex > 0 ? truncatedMessages[truncatedMessages.length - 1]?.id ?? null : null,
              responseStepsList: [],
              computerState: {
                ...threadState.computerState,
                progressTracker: [],
                progressIdMap: {},
                activeIndex: 0,
                shouldJumpToLive: true,
              },
            },
          },
        };
      });

      return updatedMessage;
    },
  }));
}
