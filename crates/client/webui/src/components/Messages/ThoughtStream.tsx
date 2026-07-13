import React from 'react';
import ReactMarkdown from 'react-markdown';
import remarkGfm from 'remark-gfm';
import { useMarkdownRenderers } from '@/components/Shared/MarkdownRenderer';

export const ThoughtStreamComponent: React.FC<{ streamingThoughts: string[] }> = ({
  streamingThoughts,
}) => {
  const components = useMarkdownRenderers(true);
  const lastThought = streamingThoughts[streamingThoughts.length - 1] ?? '';
  return (
    <ReactMarkdown remarkPlugins={[remarkGfm]} components={components as any}>
      {lastThought}
    </ReactMarkdown>
  );
};
