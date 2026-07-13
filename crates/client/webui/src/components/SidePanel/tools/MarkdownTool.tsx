import React from 'react';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { prism, vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import remarkGfm from 'remark-gfm';
import { useTheme } from '@/hooks/useTheme';

interface MarkdownToolProps {
  content: string;
}

export const MarkdownTool: React.FC<MarkdownToolProps> = ({ content }) => {
  const { colorScheme } = useTheme();
  const theme = colorScheme === 'dark' ? vscDarkPlus : prism;

  return (
    <div className="fc-h-full fc-w-full fc-overflow-auto fc-p-4">
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          code: ({ className, children }: { className?: string; children?: React.ReactNode }) => {
            const language = className ? className.replace('language-', '') : '';
            if (!className) {
              return (
                <code className="fc-px-1 fc-py-0.5 fc-rounded fc-text-sm" style={{ backgroundColor: 'var(--fc-surface-hover)' }}>
                  {children}
                </code>
              );
            }
            return (
              <SyntaxHighlighter
                language={language}
                style={theme as any}
                PreTag="div"
                customStyle={{ fontSize: '12px', borderRadius: '8px' }}
              >
                {String(children).replace(/\n$/, '')}
              </SyntaxHighlighter>
            );
          },
          h1: ({ children }: any) => (
            <h1 className="fc-text-2xl fc-font-bold fc-mt-6 fc-mb-4">{children}</h1>
          ),
          h2: ({ children }: any) => (
            <h2 className="fc-text-xl fc-font-bold fc-mt-5 fc-mb-3">{children}</h2>
          ),
          h3: ({ children }: any) => (
            <h3 className="fc-text-lg fc-font-semibold fc-mt-4 fc-mb-2">{children}</h3>
          ),
          p: ({ children }: any) => (
            <p className="fc-mb-2 fc-text-sm fc-leading-6" style={{ color: 'var(--fc-text)' }}>{children}</p>
          ),
          a: ({ href, children }: any) => (
            <a
              target="_blank"
              rel="noreferrer"
              href={href}
              className="fc-text-sm hover:fc-underline"
              style={{ color: 'var(--fc-link)' }}
            >
              {children}
            </a>
          ),
          ul: ({ children }: any) => (
            <ul className="fc-list-disc fc-list-outside fc-ml-4 fc-text-sm fc-leading-6">
              {children}
            </ul>
          ),
          ol: ({ children }: any) => (
            <ol className="fc-list-decimal fc-list-outside fc-ml-6 fc-text-sm fc-leading-6">
              {children}
            </ol>
          ),
          li: ({ children }: any) => (
            <li className="fc-mb-1 fc-text-sm" style={{ color: 'var(--fc-text)' }}>{children}</li>
          ),
          table: ({ children }: any) => (
            <div className="fc-overflow-x-auto">
              <table className="fc-border-collapse fc-border fc-w-full fc-text-sm" style={{ borderColor: 'var(--fc-border)' }}>
                {children}
              </table>
            </div>
          ),
          th: ({ children }: any) => (
            <th className="fc-border fc-px-4 fc-py-2 fc-text-left" style={{ borderColor: 'var(--fc-border)', backgroundColor: 'var(--fc-surface)' }}>
              {children}
            </th>
          ),
          td: ({ children }: any) => (
            <td className="fc-border fc-px-4 fc-py-2" style={{ borderColor: 'var(--fc-border)' }}>{children}</td>
          ),
        }}
      >
        {content}
      </ReactMarkdown>
    </div>
  );
};
