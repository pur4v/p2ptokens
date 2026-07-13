import React, { useMemo } from 'react';
import {
  BrowserToolProps,
  IframeToolProps,
  IToolType,
  PlanToolProps,
  ProgressTrackerItem,
} from '@/types';
import { useConfig } from '@/provider/ChatProvider';
import { ErrorBoundary } from '@/components/Shared/ErrorBoundary';
import { ErrorFallback } from '@/components/SidePanel/tools/ErrorFallback';
import { BrowserTool } from '@/components/SidePanel/tools/BrowserTool';
import { CodeViewerTool } from '@/components/SidePanel/tools/CodeViewerTool';
import { RunCodeTool } from '@/components/SidePanel/tools/RunCodeTool';
import { SearchEngineTool } from '@/components/SidePanel/tools/SearchEngineTool';
import { IframeTool } from '@/components/SidePanel/tools/IframeTool';
import { MarkdownTool } from '@/components/SidePanel/tools/MarkdownTool';

interface ToolContentRendererProps {
  progressItem: ProgressTrackerItem;
}

export const ToolContentRenderer: React.FC<ToolContentRendererProps> = ({ progressItem }) => {
  const config = useConfig();

  // Check for custom tool renderer first
  const CustomRenderer = config.toolRenderers?.[progressItem?.name];
  if (CustomRenderer) {
    return (
      <ErrorBoundary fallback={<ErrorFallback />}>
        <CustomRenderer progressItem={progressItem} />
      </ErrorBoundary>
    );
  }

  const content = useMemo(() => {
    const contentMapper: Record<IToolType, () => any> = {
      browser: () => (progressItem?.detail as unknown as BrowserToolProps)?.screenshot ?? '',
      search_web_tool: () =>
        (progressItem?.detail?.content as { results: any[] })?.results ?? [],
      scrape_url_tool: () =>
        (progressItem?.detail?.content as BrowserToolProps)?.screenshot ?? '',
      run_code_and_visualize_tool: () => (progressItem?.detail as any)?.code ?? '',
      list_tables_tool: () => JSON.stringify(progressItem?.detail?.content, null, 2),
      describe_table_tool: () => JSON.stringify(progressItem?.detail?.content, null, 2),
      execute_query_tool: () =>
        (progressItem?.detail?.content as { sql_query: string })?.sql_query ?? '',
      run_code_tool: () => (progressItem?.detail as any)?.code ?? '',
      get_browser_vnc_url: () => {
        const iframeContent = progressItem?.detail?.content as IframeToolProps;
        return {
          vnc_url: iframeContent?.vnc_url ?? '',
          liveExpired: iframeContent?.expiry_time
            ? iframeContent.expiry_time - Math.floor(Date.now() / 1000) < 0
            : false,
        };
      },
      create_plan_tool: () =>
        (progressItem?.detail?.content as unknown as PlanToolProps)?.plan_markdown ?? '',
      update_plan_tool: () =>
        (progressItem?.detail?.content as unknown as PlanToolProps)?.plan_markdown ?? '',
    };

    const getter = contentMapper[progressItem?.name];
    if (getter) return getter();

    // Default fallback
    const response = progressItem?.detail?.content ?? progressItem?.detail ?? 'Rendering content';
    return typeof response === 'string' ? response : JSON.stringify(response, null, 2);
  }, [progressItem]);

  const getRunCodeOutput = () => {
    if (progressItem?.name !== 'run_code_tool') return '';
    const c = progressItem?.detail?.content;
    return typeof c === 'string' ? c : JSON.stringify(c, null, 2) ?? '';
  };

  const toolMapper: Record<IToolType, React.ReactNode> = {
    browser: <BrowserTool content={content} />,
    search_web_tool: <SearchEngineTool results={content} />,
    scrape_url_tool: <BrowserTool content={content} />,
    run_code_and_visualize_tool: <CodeViewerTool content={content} language="python" />,
    list_tables_tool: <CodeViewerTool content={content} language="json" />,
    describe_table_tool: <CodeViewerTool content={content} language="json" />,
    execute_query_tool: <CodeViewerTool content={content} language="sql" />,
    run_code_tool: <RunCodeTool content={content} outputContent={getRunCodeOutput()} />,
    get_browser_vnc_url: (
      <IframeTool content={content.vnc_url} liveExpired={content.liveExpired} />
    ),
    create_plan_tool: <MarkdownTool content={content} />,
    update_plan_tool: <MarkdownTool content={content} />,
  };

  return (
    <div className="fc-w-full fc-h-full fc-relative fc-resize-none">
      <ErrorBoundary fallback={<ErrorFallback />}>
        {toolMapper[progressItem?.name as IToolType] ?? (
          <CodeViewerTool content={String(content)} />
        )}
      </ErrorBoundary>
    </div>
  );
};
