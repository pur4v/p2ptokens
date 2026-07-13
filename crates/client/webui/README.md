# @p2ptokens/chat

A self-contained, production-ready chat assistant widget that can be embedded into **any** web application. It provides a complete chat experience out of the box — message threads, streaming responses, file uploads, tool previews, theming, and more.

**Tech stack:** React 19, Zustand 5, Tailwind CSS 3 (prefixed `fc-`), Framer Motion, react-markdown, lucide-react. Built as an IIFE bundle via Vite.

**Two integration modes:**
1. **Script tag** — drop a single `<script>` tag into any HTML page (no build step needed)
2. **React import** — import individual components into your React app for full control

> For the full integration guide with detailed configuration, adapter setup, streaming protocol, theming, and examples, see [INTEGRATION_GUIDE.md](./INTEGRATION_GUIDE.md).

---

## Architecture Overview

```
+-----------------------------------------------------------+
|                     Host Application                       |
|                                                           |
|  +------------------+  +------------------------------+   |
|  |  HistorySidebar  |  |       ChatContainer          |   |
|  |  (thread list)   |  |  +------------------------+  |   |
|  |                  |  |  |    Message List         |  |   |
|  |  - New Chat btn  |  |  |  - UserMessage(s)      |  |   |
|  |  - Thread items  |  |  |  - AssistantMessage(s)  |  |   |
|  |  - Delete        |  |  +------------------------+  |   |
|  |                  |  |  |    ChatInput            |  |   |
|  |                  |  |  |  - Textarea             |  |   |
|  |                  |  |  |  - File upload          |  |   |
|  |                  |  |  |  - Send / Stop btn      |  |   |
|  |                  |  |  +------------------------+  |   |
|  +------------------+  +------------------------------+   |
|                                                           |
|  +-----------------------------------------------------+  |
|  |                SidePanel (optional)                  |  |
|  |  - Computer tool preview                            |  |
|  |  - Streaming thoughts preview                       |  |
|  |  - Custom tool renderers                            |  |
|  +-----------------------------------------------------+  |
+-----------------------------------------------------------+

State Layer (Zustand):
  - ChatStore    → messages, threads, streaming state
  - HistoryStore → thread list, concurrent thread status
  - PreviewStore → side panel preview state

Backend Layer (ChatAdapter):
  - startStream()         → streaming chat responses
  - fetchThreads()        → thread list
  - fetchThreadMessages() → load thread history
  - uploadFile()          → file uploads
  - submitReaction()      → like/dislike
```

**Key design decisions:**
- **Style isolation**: All Tailwind classes use the `fc-` prefix, and all styles are scoped under `.p2p-chat-root`. The widget never conflicts with host app styles.
- **Backend-agnostic**: All API calls go through a `ChatAdapter` interface. Use the built-in default adapter or provide your own to connect to any backend.
- **Shared state**: Multiple `render()` calls sharing the same config object reference automatically share Zustand stores, keeping sidebar and chat container in sync.

---

## Project Structure

```
packages/chat-assistant/
├── src/
│   ├── index.ts                    # Main entry — all public exports
│   ├── mount.tsx                   # Script-tag mounting logic (createRoot, shared stores)
│   ├── styles.css                  # Base CSS variables & global styles
│   ├── types/
│   │   └── index.ts                # All TypeScript interfaces & types
│   ├── provider/
│   │   ├── ChatProvider.tsx        # React context provider (config, adapter, stores)
│   │   └── adapters/
│   │       └── defaultAdapter.ts   # Built-in REST API adapter
│   ├── store/
│   │   ├── createChatStore.ts      # Chat state (messages, threads, streaming)
│   │   ├── createHistoryStore.ts   # Thread list & concurrent thread status
│   │   └── createPreviewStore.ts   # Side panel preview state
│   ├── streaming/
│   │   ├── streamingService.ts     # Main streaming orchestration
│   │   ├── chunkHandler.ts         # Parses & routes stream chunks by type
│   │   └── resumableStream.ts      # Stream resume & cancel logic
│   ├── hooks/
│   │   ├── useFileUpload.ts        # File upload state management
│   │   ├── useTheme.ts             # Light/dark/auto theme detection
│   │   └── useThemeVars.ts         # ChatThemeConfig → CSS variables
│   ├── components/
│   │   ├── registry.ts             # Maps component names to React components
│   │   ├── ChatContainer/
│   │   │   └── ChatContainer.tsx   # Full chat UI (messages + input + side panel)
│   │   ├── Input/
│   │   │   ├── ChatInput.tsx       # Chat textarea with send/stop/file upload
│   │   │   ├── FilePreview.tsx     # Uploaded file thumbnails
│   │   │   └── FileUploadButton.tsx # File picker button
│   │   ├── History/
│   │   │   ├── HistorySidebar.tsx  # Thread list sidebar
│   │   │   ├── ThreadItem.tsx      # Single thread in the list
│   │   │   └── ThreadStatusIcon.tsx # Running/completed status indicator
│   │   ├── Messages/
│   │   │   ├── AssistantMessage.tsx # AI response with markdown, tools, reactions
│   │   │   ├── UserMessage.tsx     # User message bubble with edit support
│   │   │   ├── FollowUpQuestions.tsx # Suggested follow-up questions
│   │   │   ├── LikeDislike.tsx     # Thumbs up/down buttons
│   │   │   ├── ResponseStepsList.tsx # Tool execution steps display
│   │   │   └── ThoughtStream.tsx   # Streaming thoughts display
│   │   ├── Shared/
│   │   │   ├── AutoScrollContainer.tsx # Auto-scrolls to latest message
│   │   │   ├── CopyContent.tsx     # Copy-to-clipboard button
│   │   │   ├── ErrorBoundary.tsx   # React error boundary
│   │   │   ├── MarkdownRenderer.tsx # Custom markdown renderers (code blocks, tables, etc.)
│   │   │   └── ProgressMessage.tsx # Animated "Thinking..." indicator
│   │   └── SidePanel/
│   │       ├── SidePanel.tsx       # Side panel container with animations
│   │       ├── ComputerToolPreview.tsx # Computer/browser tool preview
│   │       ├── ProgressPreview.tsx  # Progress timeline with scrubbing
│   │       ├── StreamingThoughtsPreview.tsx # Full thoughts viewer
│   │       ├── ToolContentRenderer.tsx # Routes tool content to renderers
│   │       ├── ToolHeader.tsx      # Tool preview header
│   │       ├── ToolTop.tsx         # Tool preview top bar
│   │       └── tools/
│   │           ├── BrowserTool.tsx  # Browser screenshot viewer
│   │           ├── CodeViewerTool.tsx # Code with syntax highlighting
│   │           ├── RunCodeTool.tsx  # Code execution results
│   │           ├── SearchEngineTool.tsx # Search results display
│   │           ├── IframeTool.tsx   # Iframe/VNC preview
│   │           ├── MarkdownTool.tsx # Markdown preview
│   │           └── ErrorFallback.tsx # Error fallback for tool rendering
│   └── utils/
│       ├── constants.ts            # Chunk delimiter, defaults
│       └── helpers.ts              # Utility functions (createNewChat)
├── examples/
│   └── index.html                  # Working HTML example
├── package.json
├── vite.config.ts                  # Build config (IIFE bundle)
├── tsconfig.json
├── tailwind.config.ts              # Tailwind with fc- prefix
└── postcss.config.js
```

---

## Build & Output

```bash
npm install       # Install dependencies
npm run build     # Build production bundle
npm run dev       # Start dev server
npm run type-check # Type-check without emitting
```

Produces: **`dist/chat.min.js`**
- Format: IIFE (works in any browser, no bundler needed)
- Global: `P2PTokensChat`
- CSS: inlined (no separate file)
- Source maps: `dist/chat.min.js.map`

---

## Quick Start

### Script tag (any HTML page)

```html
<div id="chat" style="height: 100vh;"></div>
<script src="path/to/chat.min.js"></script>
<script>
  P2PTokensChat.render('ChatContainer', '#chat', {
    apiBaseUrl: 'https://your-api.example.com/v1.0',
    getAuthHeaders: () => ({ 'Authorization': 'Bearer TOKEN' }),
  });
</script>
```

### React app

```tsx
import { ChatProvider, ChatContainer, HistorySidebar } from '@p2ptokens/chat';

function App() {
  const config = {
    apiBaseUrl: 'https://your-api.example.com/v1.0',
    getAuthHeaders: () => ({ 'Authorization': 'Bearer TOKEN' }),
  };

  return (
    <ChatProvider config={config}>
      <div style={{ display: 'flex', height: '100vh' }}>
        <HistorySidebar />
        <ChatContainer />
      </div>
    </ChatProvider>
  );
}
```

---

## Available Components

| Registry Name | Import Name | Description |
|---------------|-------------|-------------|
| `'ChatContainer'` | `ChatContainer` | Full chat UI: messages list + input + side panel |
| `'ChatInput'` | `ChatInput` | Standalone chat input with file upload, send/stop |
| `'History'` | `HistorySidebar` | Thread list sidebar with new chat button |
| `'AssistantMessage'` | `AssistantMessage` | Single assistant message with markdown, tools, reactions |
| `'UserMessage'` | `UserMessage` | Single user message bubble with edit support |
| `'SidePanel'` | `SidePanel` | Tool preview side panel (computer tool, thoughts, custom) |

Additional tool components (React import only): `BrowserTool`, `CodeViewerTool`, `RunCodeTool`, `SearchEngineTool`, `IframeTool`, `MarkdownTool`

---

## Dependencies

### Runtime

| Package | Version | Purpose |
|---------|---------|---------|
| `react` | ^19.0.0 | UI framework |
| `react-dom` | ^19.0.0 | DOM rendering |
| `zustand` | ^5.0.3 | State management (3 stores) |
| `framer-motion` | ^12.6.3 | Side panel animations |
| `lucide-react` | ^0.474.0 | Icons (Send, Stop, Plus, Brain, etc.) |
| `react-markdown` | ^10.1.0 | Markdown rendering in messages |
| `react-syntax-highlighter` | ^15.6.1 | Code block syntax highlighting |
| `react-textarea-autosize` | ^8.5.7 | Auto-growing chat input textarea |
| `remark-gfm` | ^4.0.1 | GitHub Flavored Markdown support |
| `clsx` | ^2.1.1 | Conditional CSS class names |
| `echarts` | ^5.6.0 | Chart rendering |
| `echarts-for-react` | ^3.0.2 | React wrapper for ECharts |

### Dev

| Package | Version | Purpose |
|---------|---------|---------|
| `@vitejs/plugin-react` | ^4.3.4 | Vite React plugin |
| `tailwindcss` | ^3.4.17 | Utility-first CSS (with `fc-` prefix) |
| `autoprefixer` | ^10.4.20 | CSS vendor prefixing |
| `postcss` | ^8.5.3 | CSS processing |
| `typescript` | ^5.7.3 | Type checking |
| `vite` | ^6.2.0 | Build tool (IIFE output) |

---

## Further Reading

- **[INTEGRATION_GUIDE.md](./INTEGRATION_GUIDE.md)** — Full integration guide with:
  - Detailed configuration reference (ChatCommonConfig, ChatThemeConfig, ChatTextConfig, component configs)
  - Backend adapter setup (ChatAdapter interface, default endpoints, custom adapter examples)
  - Streaming protocol (chunk types, lifecycle, auto-resume)
  - Theming & styling (CSS variables, dark mode, style isolation)
  - Custom tool renderers
  - State management (Zustand stores, hooks)
  - TypeScript types reference
  - 5 complete working examples
