import React, { memo, useCallback, useMemo, useRef } from 'react';
import { Code, Copy, Download } from 'lucide-react';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { prism, vscDarkPlus } from 'react-syntax-highlighter/dist/esm/styles/prism';
import { useTheme } from '@/hooks/useTheme';

const MemoizedChartRenderer = memo(({ chartContent }: { chartContent: string }) => {
  const chartData = useMemo(() => {
    const trimmed = chartContent.trim();
    if (trimmed.startsWith('{') && trimmed.endsWith('}')) {
      try {
        return JSON.parse(trimmed);
      } catch {
        return null;
      }
    }
    return null;
  }, [chartContent]);

  if (!chartData) {
    return (
      <div className="fc-my-4 fc-p-4 fc-border fc-border-gray-200 fc-rounded-lg fc-animate-pulse fc-h-48 fc-bg-gray-50" />
    );
  }

  return (
    <div className="fc-my-4 fc-p-4 fc-border fc-border-gray-200 fc-rounded-lg">
      <pre className="fc-text-xs fc-overflow-auto">{JSON.stringify(chartData, null, 2)}</pre>
    </div>
  );
});
MemoizedChartRenderer.displayName = 'MemoizedChartRenderer';

export const useMarkdownRenderers = (lightStreamingThoughts: boolean = false) => {
  const streamingTextClass = lightStreamingThoughts ? 'fc-text-gray-400 fc-opacity-70' : '';
  const streamingContainerClass = lightStreamingThoughts ? 'fc-opacity-70' : '';

  const components = useMemo(
    () => ({
      code: ({ className, children }: { className?: string; children?: React.ReactNode }) => {
        const { colorScheme } = useTheme();

        if (children && typeof children === 'string') {
          const lines = children.split('\n');
          const idx = lines.findIndex((l) => l.includes('[fileName]'));
          if (idx !== -1) {
            lines.splice(idx, 1);
            children = lines.join('\n').trim();
          }
        }

        const language = className ? className.replace('language-', '') : '';
        const codeContent = String(children).replace(/\n$/, '');
        const codeFontSize = lightStreamingThoughts ? '11px' : '12px';
        const codeLineHeight = lightStreamingThoughts ? '13px' : '14px';

        const handleCopy = useCallback(() => {
          navigator.clipboard.writeText(codeContent).catch(() => {});
        }, [codeContent]);

        const handleDownload = useCallback(() => {
          const extMap: Record<string, string> = {
            python: 'py', javascript: 'js', typescript: 'ts', java: 'java',
            cpp: 'cpp', c: 'c', ruby: 'rb', go: 'go', rust: 'rs',
            php: 'php', sql: 'sql', html: 'html', css: 'css', json: 'json',
            yaml: 'yaml', shell: 'sh', bash: 'sh', markdown: 'md',
          };
          const ext = extMap[language.toLowerCase()] || 'txt';
          const blob = new Blob([codeContent], { type: 'text/plain' });
          const url = URL.createObjectURL(blob);
          const a = document.createElement('a');
          a.href = url;
          a.download = `code.${ext}`;
          document.body.appendChild(a);
          a.click();
          document.body.removeChild(a);
          URL.revokeObjectURL(url);
        }, [codeContent, language]);

        if (language === 'chart') {
          return <MemoizedChartRenderer chartContent={codeContent} />;
        }

        return className ? (
          <div className={`fc-relative fc-z-[1] fc-cursor-default ${streamingContainerClass}`}>
            <div className="fc-flex fc-flex-col fc-bg-gray-50 fc-border fc-border-gray-200 fc-rounded-lg fc-overflow-hidden">
              <div className="fc-flex fc-items-center fc-justify-between fc-px-3 fc-py-2 fc-border-b fc-border-gray-200">
                <div className={`fc-flex fc-items-center fc-gap-2 fc-text-gray-600 ${lightStreamingThoughts ? 'fc-text-[10px] md:fc-text-[11px]' : 'fc-text-xs fc-font-medium'} ${streamingTextClass}`}>
                  <Code size={lightStreamingThoughts ? 14 : 16} />
                  <span>{language || 'Code'} Code</span>
                </div>
                <div className="fc-flex fc-items-center fc-gap-2">
                  <button type="button" onClick={handleCopy} className="fc-p-1 fc-rounded hover:fc-bg-gray-200 fc-transition-colors fc-text-gray-500 fc-bg-transparent fc-border-0 fc-cursor-pointer" title="Copy code">
                    <Copy size={lightStreamingThoughts ? 14 : 16} />
                  </button>
                  <button type="button" onClick={handleDownload} className="fc-p-1 fc-rounded hover:fc-bg-gray-200 fc-transition-colors fc-text-gray-500 fc-bg-transparent fc-border-0 fc-cursor-pointer" title="Download code">
                    <Download size={lightStreamingThoughts ? 14 : 16} />
                  </button>
                </div>
              </div>
              <div className={`fc-bg-gray-50 fc-overflow-x-auto ${streamingContainerClass}`}>
                <SyntaxHighlighter
                  language={language}
                  style={colorScheme === 'light' ? prism : vscDarkPlus}
                  PreTag="div"
                  codeTagProps={{
                    style: {
                      background: 'none', margin: '0px', fontSize: codeFontSize,
                      lineHeight: codeLineHeight, whiteSpace: 'pre-wrap',
                      wordBreak: 'break-all', overflowWrap: 'break-word', maxWidth: '100%',
                    },
                  }}
                  customStyle={{
                    margin: 0, padding: lightStreamingThoughts ? '8px' : '12px',
                    fontSize: codeFontSize, background: 'none',
                  }}
                >
                  {codeContent}
                </SyntaxHighlighter>
              </div>
            </div>
          </div>
        ) : (
          <code className={`fc-bg-gray-100 fc-p-1 fc-rounded fc-break-words fc-overflow-x-auto fc-max-w-full fc-inline-flex ${lightStreamingThoughts ? 'fc-text-[10px] md:fc-text-[11px]' : 'fc-text-sm'} ${streamingTextClass}`}>
            {children}
          </code>
        );
      },
      h1: ({ children }: any) => <h1 className={`${lightStreamingThoughts ? 'fc-text-lg md:fc-text-2xl fc-my-3 fc-font-normal' : 'fc-text-3xl fc-my-4 fc-font-extrabold'} fc-leading-tight ${streamingTextClass}`}>{children}</h1>,
      h2: ({ children }: any) => <h2 className={`${lightStreamingThoughts ? 'fc-text-base md:fc-text-2xl fc-my-3 fc-font-semibold' : 'fc-text-2xl fc-my-4 fc-font-bold'} fc-leading-snug ${streamingTextClass}`}>{children}</h2>,
      h3: ({ children }: any) => <h3 className={`${lightStreamingThoughts ? 'fc-text-sm md:fc-text-xl fc-my-2 fc-font-medium' : 'fc-text-xl fc-mt-4 fc-mb-3 fc-font-semibold'} fc-leading-snug ${streamingTextClass}`}>{children}</h3>,
      h4: ({ children }: any) => <h4 className={`${lightStreamingThoughts ? 'fc-text-xs md:fc-text-lg fc-font-medium' : 'fc-text-lg fc-font-semibold'} fc-mt-4 fc-mb-2 fc-leading-snug ${streamingTextClass}`}>{children}</h4>,
      h5: ({ children }: any) => <h5 className={`${lightStreamingThoughts ? 'fc-text-[11px] md:fc-text-base fc-font-normal' : 'fc-text-base fc-font-medium'} fc-leading-snug fc-my-2 ${streamingTextClass}`}>{children}</h5>,
      h6: ({ children }: any) => <h6 className={`${lightStreamingThoughts ? 'fc-text-[10px] md:fc-text-sm fc-font-normal' : 'fc-text-sm fc-font-medium'} fc-my-2 fc-leading-snug ${streamingTextClass}`}>{children}</h6>,
      p: ({ children }: any) => <p className={`fc-mb-2 fc-text-gray-900 ${lightStreamingThoughts ? 'fc-text-[10px] md:fc-text-[13px] fc-leading-4' : 'fc-text-sm fc-leading-6'} fc-break-words fc-overflow-x-auto fc-max-w-full ${streamingTextClass}`}>{children}</p>,
      strong: ({ children }: any) => <strong className={`fc-mb-2 fc-text-gray-900 ${lightStreamingThoughts ? 'fc-text-[10px] md:fc-text-[13px] fc-leading-4' : 'fc-text-sm fc-leading-6'} ${streamingTextClass}`}>{children}</strong>,
      blockquote: ({ children }: any) => <blockquote className={`fc-border-l-4 fc-rounded fc-border-gray-300 fc-pl-4 fc-italic fc-text-gray-700 ${lightStreamingThoughts ? 'fc-text-[10px] md:fc-text-[13px] fc-leading-4' : 'fc-text-sm fc-leading-6'} ${streamingTextClass}`}>{children}</blockquote>,
      a: ({ href, children }: any) => <a target="_blank" rel="noreferrer" href={href as string} className={`fc-text-blue-500 hover:fc-underline fc-break-words fc-break-all fc-whitespace-normal ${lightStreamingThoughts ? 'fc-text-[10px] md:fc-text-[13px] fc-leading-4' : 'fc-text-sm fc-leading-6'} ${streamingTextClass}`}>{children}</a>,
      ul: ({ children }: any) => <ul className={`fc-list-disc fc-list-outside fc-ml-4 ${lightStreamingThoughts ? 'fc-text-[10px] md:fc-text-[13px] fc-leading-4' : 'fc-text-sm fc-leading-6'} ${streamingTextClass}`}>{children}</ul>,
      ol: ({ children }: any) => <ol className={`fc-list-decimal fc-list-outside fc-ml-6 ${lightStreamingThoughts ? 'fc-text-[10px] md:fc-text-[13px] fc-leading-4' : 'fc-text-sm fc-leading-6'} ${streamingTextClass}`}>{children}</ol>,
      li: ({ children }: any) => <li className={`fc-mb-2 fc-text-gray-900 ${lightStreamingThoughts ? 'fc-text-[10px] md:fc-text-[13px] fc-leading-4' : 'fc-text-sm fc-leading-6'} ${streamingTextClass}`}>{children}</li>,
      table: ({ children }: any) => (
        <div className={`fc-overflow-x-auto ${streamingContainerClass}`}>
          <table className={`fc-border-collapse fc-border fc-border-gray-200 fc-w-full ${lightStreamingThoughts ? 'fc-text-[10px] md:fc-text-[13px] fc-leading-4' : 'fc-text-sm fc-leading-6'}`}>{children}</table>
        </div>
      ),
      th: ({ children }: any) => <th className={`fc-border fc-border-gray-200 ${lightStreamingThoughts ? 'fc-px-2 fc-py-1' : 'fc-px-4 fc-py-2'} fc-bg-gray-50 fc-text-left ${streamingTextClass}`}>{children}</th>,
      td: ({ children }: any) => <td className={`fc-border fc-border-gray-200 ${lightStreamingThoughts ? 'fc-px-2 fc-py-1' : 'fc-px-4 fc-py-2'} ${streamingTextClass}`}>{children}</td>,
      img: ({ src, alt }: { src: string; alt?: string }) => <img className={`fc-max-w-full fc-h-auto fc-rounded ${streamingContainerClass}`} src={src} alt={alt || 'Image'} />,
    }),
    [streamingTextClass, streamingContainerClass, lightStreamingThoughts]
  );

  return components;
};
