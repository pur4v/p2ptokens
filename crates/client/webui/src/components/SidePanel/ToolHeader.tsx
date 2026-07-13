import React from 'react';
import {
  Code,
  Globe,
  CircleArrowOutUpLeft,
  Table,
  Ratio,
  DatabaseZap,
  PenTool,
  Monitor,
} from 'lucide-react';
import { IToolHeaderMessage, IToolType } from '@/types';

interface ToolHeaderProps {
  tool: IToolType;
  message: IToolHeaderMessage;
}

const toolNameMapper: Record<IToolType, string> = {
  browser: 'Web Browser',
  search_web_tool: 'Search Engine',
  scrape_url_tool: 'Browser',
  run_code_and_visualize_tool: 'Terminal',
  list_tables_tool: 'Table Lister',
  describe_table_tool: 'Table Describer',
  execute_query_tool: 'Query Executor',
  run_code_tool: 'Code Runner',
  get_browser_vnc_url: 'Browser',
  create_plan_tool: 'Plan Creator',
  update_plan_tool: 'Plan Updater',
};

const toolIconMapper: Record<IToolType, React.ReactNode> = {
  browser: <Monitor size={16} />,
  search_web_tool: <Globe size={16} />,
  scrape_url_tool: <CircleArrowOutUpLeft size={16} />,
  run_code_and_visualize_tool: <Code size={16} />,
  list_tables_tool: <Table size={16} />,
  describe_table_tool: <Ratio size={16} />,
  execute_query_tool: <DatabaseZap size={16} />,
  run_code_tool: <Code size={16} />,
  get_browser_vnc_url: <Monitor size={16} />,
  create_plan_tool: <PenTool size={16} />,
  update_plan_tool: <PenTool size={16} />,
};

export const ToolHeader: React.FC<ToolHeaderProps> = ({ tool, message }) => (
  <div className="fc-px-4 fc-overflow-hidden fc-min-h-[48px]">
    <div className="fc-flex fc-items-center fc-w-full">
      <div className="fc-p-2 fc-rounded-lg" style={{ backgroundColor: 'var(--fc-surface)' }}>
        <div className="fc-border-2 fc-p-1 fc-rounded-md" style={{ borderColor: 'var(--fc-border)', color: 'var(--fc-text)' }}>
          {toolIconMapper[tool] ?? <PenTool size={16} />}
        </div>
      </div>
      <div className="fc-ml-2">
        <div className="fc-text-sm fc-mb-1 fc-flex fc-gap-1">
          <span style={{ color: 'var(--fc-text-secondary)' }}>Using</span>
          <span style={{ color: 'var(--fc-text)' }}>{toolNameMapper[tool] ?? 'Agent Tool'}</span>
        </div>
        {(message.title || message.description) && (
          <div className="fc-text-xs fc-flex fc-gap-1 fc-px-2.5 fc-py-1 fc-rounded-full fc-line-clamp-1 fc-w-fit fc-max-w-[90%]" style={{ backgroundColor: 'var(--fc-surface)' }}>
            {message.title && (
              <span className="fc-whitespace-nowrap" style={{ color: 'var(--fc-text-secondary)' }}>{message.title}</span>
            )}
            {message.description && (
              <span className="fc-px-1 fc-truncate fc-max-w-[200px]" style={{ color: 'var(--fc-text-secondary)' }}>
                {message.description}
              </span>
            )}
          </div>
        )}
      </div>
    </div>
  </div>
);
