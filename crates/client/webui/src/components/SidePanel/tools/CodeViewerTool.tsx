import React, { useCallback } from 'react';
import { Copy } from 'lucide-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { prism, vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { useTheme } from '@/hooks/useTheme';

interface CodeViewerToolProps {
  content: string;
  language?: string;
  className?: string;
}

export const CodeViewerTool = React.memo<CodeViewerToolProps>(
  ({ content, language = 'javascript', className = '' }) => {
    const { colorScheme } = useTheme();

    const handleCopy = useCallback(() => {
      navigator.clipboard.writeText(content).catch(() => {});
    }, [content]);

    return (
      <div className={`fc-h-full fc-w-full fc-flex fc-flex-col ${className}`}>
        <div className="fc-flex fc-items-center fc-justify-between fc-px-3 fc-py-2 fc-border-b" style={{ borderColor: 'var(--fc-border)', backgroundColor: 'var(--fc-surface)' }}>
          <span className="fc-text-xs fc-font-medium" style={{ color: 'var(--fc-text-secondary)' }}>{language}</span>
          <button
            type="button"
            onClick={handleCopy}
            className="fc-p-1 fc-rounded fc-transition-colors fc-bg-transparent fc-border-0 fc-cursor-pointer"
            style={{ color: 'var(--fc-text-secondary)' }}
            title="Copy code"
          >
            <Copy size={14} />
          </button>
        </div>
        <div className="fc-flex-1 fc-overflow-auto">
          <SyntaxHighlighter
            language={language}
            style={colorScheme === 'dark' ? vscDarkPlus : prism}
            PreTag="div"
            showLineNumbers
            wrapLongLines
            customStyle={{
              margin: 0,
              padding: '12px',
              fontSize: '12px',
              lineHeight: '18px',
              background: 'none',
              height: '100%',
            }}
          >
            {content}
          </SyntaxHighlighter>
        </div>
      </div>
    );
  }
);

CodeViewerTool.displayName = 'CodeViewerTool';
