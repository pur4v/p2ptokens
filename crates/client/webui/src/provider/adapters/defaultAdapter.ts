import {
  ChatAdapter,
  ChatConfig,
  FetchThreadsParams,
  FetchThreadsResponse,
  StartStreamParams,
  SuggestionResponse,
  ThreadMessagesResponse,
  UploadedImage,
  MessageReaction,
} from '@/types';

export function createDefaultAdapter(config: ChatConfig): ChatAdapter {
  const { apiBaseUrl, getAuthHeaders } = config;

  async function fetchWithAuth(
    path: string,
    init: RequestInit = {}
  ): Promise<Response> {
    const headers = {
      'Content-Type': 'application/json',
      ...getAuthHeaders(),
      ...(init.headers as Record<string, string> || {}),
    };
    const res = await fetch(`${apiBaseUrl}${path}`, { ...init, headers });
    if (!res.ok) {
      throw new Error(`HTTP ${res.status}: ${res.statusText}`);
    }
    return res;
  }

  return {
    async startStream(params: StartStreamParams): Promise<ReadableStream> {
      const body = {
        message: { content: params.message },
        product: params.productMode,
        parent_message_id: params.parentMessageId,
        stream: true,
        thread_id: params.isNewChat ? null : params.threadId,
        metadata: {
          response_length: 400,
          selected_kbs:
            params.selectedKbs && params.selectedKbs.length > 0 ? params.selectedKbs : [],
          response_format: params.outputResponseFormat || '',
        },
        images: params.images ?? [],
      };

      const headers: Record<string, string> = {
        'Content-Type': 'application/json',
        ...getAuthHeaders(),
        ...(params.extraHeaders || {}),
      };

      const res = await fetch(`${apiBaseUrl}/streaming/chat/start`, {
        method: 'POST',
        headers,
        body: JSON.stringify(body),
        signal: params.isNewChat ? undefined : undefined,
      });

      if (!res.ok) {
        throw new Error(`Stream request failed: ${res.status}`);
      }

      if (!res.body) {
        throw new Error('Response stream is null');
      }

      return res.body;
    },

    async cancelStream(threadId: string): Promise<void> {
      await fetchWithAuth(`/streaming/cancel/${threadId}/task`, { method: 'POST' });
    },

    async resumeStream(threadId: string): Promise<ReadableStream> {
      const res = await fetch(`${apiBaseUrl}/streaming/chunks/${threadId}`, {
        headers: {
          ...getAuthHeaders(),
        },
      });
      if (!res.ok) {
        throw new Error(`Resume stream failed: ${res.status}`);
      }
      if (!res.body) {
        throw new Error('Response stream is null');
      }
      return res.body;
    },

    async fetchThreads(params: FetchThreadsParams): Promise<FetchThreadsResponse> {
      const product = params.product || config.productMode || 'devas';
      const query = params.query || '';
      const res = await fetchWithAuth(
        `/threads/?page=${params.page}&user_email=${params.email}&product=${product}&page_size=25&query=${query}&agent_type=conversational`
      );
      const json = await res.json();
      return json.data;
    },

    async deleteThread(threadId: string): Promise<void> {
      await fetchWithAuth(`/threads/${threadId}`, { method: 'DELETE' });
    },

    async fetchThreadMessages(
      threadId: string,
      signal?: AbortSignal
    ): Promise<ThreadMessagesResponse> {
      const res = await fetchWithAuth(`/threads/thread/${threadId}/messages`, { signal });
      const json = await res.json();
      return json;
    },

    async fetchSuggestions(
      query: string,
      limit: number,
      signal?: AbortSignal
    ): Promise<SuggestionResponse> {
      const res = await fetchWithAuth(
        `/prompts/suggestions?query=${encodeURIComponent(query)}&limit=${limit}`,
        { signal }
      );
      return res.json();
    },

    async uploadFile(file: File): Promise<UploadedImage> {
      const formData = new FormData();
      formData.append('file', file);
      const headers = { ...getAuthHeaders() };
      const res = await fetch(`${apiBaseUrl}/upload`, {
        method: 'POST',
        headers,
        body: formData,
      });
      if (!res.ok) throw new Error(`Upload failed: ${res.status}`);
      const json = await res.json();
      return json.data;
    },

    async submitReaction(
      threadId: string,
      messageId: string,
      reaction: MessageReaction
    ): Promise<void> {
      await fetchWithAuth(`/threads/${threadId}/messages/${messageId}/reaction`, {
        method: 'POST',
        body: JSON.stringify(reaction),
      });
    },
  };
}
