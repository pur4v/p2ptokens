// ============================================================
// Theme Config — colors, spacing, radii
// ============================================================

export interface ChatThemeConfig {
  primaryColor?: string;
  primaryHoverColor?: string;
  userBubbleColor?: string;
  userBubbleBorder?: string;
  assistantBorderColor?: string;
  backgroundColor?: string;
  textColor?: string;
  secondaryTextColor?: string;
  borderColor?: string;
  surfaceColor?: string;
  surfaceHoverColor?: string;
  successColor?: string;
  errorColor?: string;
  warningColor?: string;
  linkColor?: string;
  codeBackground?: string;
  fontFamily?: string;
  messagePaddingX?: string;
  messagePaddingY?: string;
  messageGap?: string;
  containerPadding?: string;
  borderRadius?: string;
  inputBorderRadius?: string;
}

// ============================================================
// Text Config — all user-facing strings
// ============================================================

export interface ChatTextConfig {
  inputPlaceholder?: string;
  newChatButtonText?: string;
  emptyStateTitle?: string;
  emptyStateSubtitle?: string;
  loadingText?: string;
  errorText?: string;
  thinkingText?: string;
  answerCompletedText?: string;
  helpfulText?: string;
  followUpTitle?: string;
  noConversationsText?: string;
  feedbackText?: string;
  saveAndSendText?: string;
  cancelText?: string;
  stepsCompletedText?: string;
  stepsErrorText?: string;
  processingStepsText?: string;
  showDetailsText?: string;
  hideDetailsText?: string;
  previewText?: string;
  liveText?: string;
  copyCodeText?: string;
  downloadCodeText?: string;
  somethingWentWrongText?: string;
  noThoughtsText?: string;
}

// ============================================================
// Common Config — shared by ALL components (passed as 3rd arg)
// ============================================================

export interface ChatCommonConfig {
  apiBaseUrl: string;
  getAuthHeaders: () => Record<string, string>;
  productMode?: ProductMode;
  enableToolPreview?: boolean;
  enableFileUpload?: boolean;
  enableSuggestions?: boolean;
  enableReactions?: boolean;
  enableMessageEdit?: boolean;
  maxRetryCount?: number;
  streamChunkDelimiter?: string;
  onNavigate?: (path: string) => void;
  onThreadChange?: (threadId: string | null) => void;
  toolRenderers?: Record<string, React.ComponentType<any>>;
  theme?: 'light' | 'dark' | 'auto';
  themeConfig?: ChatThemeConfig;
  textConfig?: ChatTextConfig;
}

// Backward compatibility alias
export type ChatConfig = ChatCommonConfig;

// ============================================================
// Component-Specific Configs — passed as 4th arg to render()
// ============================================================

export interface ChatContainerConfig {
  enableComputerTool?: boolean;
  emptyStateTitle?: string;
  emptyStateSubtitle?: string;
}

export interface ChatInputComponentConfig {
  placeholder?: string;
  maxRows?: number;
  minRows?: number;
  maxFileUploads?: number;
  allowedFileTypes?: string[];
}

export interface HistoryComponentConfig {
  sidebarWidth?: string;
  showDeleteButton?: boolean;
  showSearch?: boolean;
  newChatButtonText?: string;
  noConversationsText?: string;
}

export interface MessagesComponentConfig {
  showCopyButton?: boolean;
  maxUserMessageWidth?: string;
}

export type ComponentConfig =
  | ChatContainerConfig
  | ChatInputComponentConfig
  | HistoryComponentConfig
  | MessagesComponentConfig
  | Record<string, any>;

// ============================================================
// Chat Adapter - backend abstraction
// ============================================================

export interface ChatAdapter {
  startStream(params: StartStreamParams): Promise<ReadableStream>;
  cancelStream(threadId: string): Promise<void>;
  resumeStream(threadId: string): Promise<ReadableStream>;
  fetchThreads(params: FetchThreadsParams): Promise<FetchThreadsResponse>;
  deleteThread(threadId: string): Promise<void>;
  fetchThreadMessages(threadId: string, signal?: AbortSignal): Promise<ThreadMessagesResponse>;
  fetchSuggestions?(query: string, limit: number, signal?: AbortSignal): Promise<SuggestionResponse>;
  uploadFile?(file: File): Promise<UploadedImage>;
  submitReaction?(
    threadId: string,
    messageId: string,
    reaction: MessageReaction
  ): Promise<void>;
}

export interface StartStreamParams {
  message: string;
  threadId: string | null;
  isNewChat: boolean;
  productMode: ProductMode;
  parentMessageId: string | null;
  images?: UploadedImage[];
  selectedKbs?: SelectedKB[];
  outputResponseFormat?: string;
  extraHeaders?: Record<string, string>;
}

export interface FetchThreadsParams {
  page: number;
  email: string;
  query?: string;
  product?: string;
}

export interface FetchThreadsResponse {
  threads: IThread[];
  pagination: {
    has_next: boolean;
    page: number;
    total: number;
  };
}

export interface ThreadMessagesResponse {
  success: boolean;
  data: {
    messages: any[];
    active_task: boolean;
  };
}

export interface SuggestionResponse {
  success: boolean;
  data: {
    suggestions: string[];
  };
}

// ============================================================
// Message Types
// ============================================================

export interface IAssistantChunks {
  id: string;
  type?: string;
  call_id?: string;
  action?: string;
  param?: string;
  tip?: string;
  content?: string;
  name?: string;
  detail: {
    toolUsed: any[];
  };
  stepProgress?: string;
  chunkObject?: any;
  run_id?: string;
}

export interface IAssistantMessage {
  role: 'assistant';
  id: string;
  assistantChunks: IAssistantChunks[];
  reaction?: MessageReaction;
}

export interface IUserMessage {
  role: 'user';
  id: string;
  content: string;
  images?: UploadedImage[];
  contextModuleSelection?: string;
  outputResponseFormat?: string;
  selected_kbs?: SelectedKB[];
}

export interface MessageReaction {
  type: 'liked' | 'disliked';
  dislike_reason_id: number | null;
  details: string | null;
}

export interface UploadedImage {
  type: string;
  filename: string;
  url: string;
}

export interface SelectedKB {
  kb_id: string;
  name: string;
}

// ============================================================
// Thread Types
// ============================================================

export interface IThread {
  id: string;
  title: string;
  last_message_id?: string;
  created_at?: string;
  updated_at?: string;
  user_id?: string;
  product?: string;
  is_running?: boolean;
}

export interface IThreadState {
  messages: (IAssistantMessage | IUserMessage)[];
  lastMessageId: string | null;
  title: string;
  abortController: AbortController | null;
  progressMessage: string | null;
  streaming: boolean;
  computerState: IComputerState;
  responseStepsList: any[];
  moduleOutputObject: ModuleOutputObject;
}

export interface IComputerState {
  activeIndex: number;
  shouldJumpToLive: boolean;
  progressTracker: ProgressTrackerItem[];
  progressIdMap: Record<string, number>;
}

export interface ModuleOutputObject {
  output: string;
  selected_kbs: SelectedKB[];
}

export type ProductMode = 'devas' | 'dofle';

// ============================================================
// Tool Types
// ============================================================

export type IToolType =
  | 'browser'
  | 'search_web_tool'
  | 'scrape_url_tool'
  | 'run_code_and_visualize_tool'
  | 'list_tables_tool'
  | 'describe_table_tool'
  | 'execute_query_tool'
  | 'run_code_tool'
  | 'get_browser_vnc_url'
  | 'create_plan_tool'
  | 'update_plan_tool';

export interface IToolHeaderMessage {
  title?: string;
  description?: string;
}

export interface ProgressTrackerItem {
  name: IToolType;
  message: {
    action: string;
    param: string;
  };
  call_id: string;
  detail:
    | {
        textEditor?: {
          content: string;
          oldContent: string;
          path: string;
        };
        terminal?: {
          output: string[];
          shellId: string;
        };
        browser?: {
          screenshot: string;
          url: string;
        };
        code?: string;
        content?:
          | BrowserToolProps
          | any[]
          | { results: any }
          | {
              sql_query: string;
              result: {
                total_vendors: number;
                total_styles: number;
                total_purchase_orders: number;
                total_stores_dcs: number;
                month: number;
              }[];
            }
          | {
              display_data: string;
              format_outputs: {
                format: string;
                content: {
                  type: string;
                  title: string;
                  elements: any[];
                };
              }[];
            }
          | IframeToolProps;
      }
    | BrowserToolProps;
}

export interface BrowserToolProps {
  content: string;
  contentType: string;
  screenshot: string;
}

export interface IframeToolProps {
  vnc_url: string;
  success: boolean;
  sandbox_id: string;
  message: string;
  contentType: string;
  expiry_time: number;
}

export interface PlanToolProps {
  plan_markdown: string;
  plan_type: string;
}

export interface SearchResult {
  title: string;
  url: string;
  description: string;
  result_number: number;
}

// ============================================================
// Preview Types
// ============================================================

export type AgentPreviewObjectWithParams = null | {
  title: string;
  description?: string;
  image?: string;
  textContent?: string;
  streamingThoughts?: string[];
  type?: string;
  images?: UploadedImage[];
  pptKey?: string;
  pptSerialNo?: number;
};

// ============================================================
// Concurrent Thread Status
// ============================================================

export enum ConcurrentThreadStatus {
  RUNNING = 'running',
  ACTIVE = 'active',
  COMPLETED = 'completed',
  PENDING_SEEN = 'pendingSeen',
}
