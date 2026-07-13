import React from 'react';
import { SearchResult } from '@/types';

interface SearchEngineToolProps {
  results?: SearchResult[];
}

export const SearchEngineTool: React.FC<SearchEngineToolProps> = ({ results = [] }) => {
  return (
    <div className="fc-font-sans fc-mx-auto fc-p-2 fc-h-full fc-overflow-auto" style={{ backgroundColor: 'var(--fc-surface-hover)' }}>
      <div className="fc-flex fc-flex-col fc-gap-2">
        {results.map((result, index) => (
          <div
            key={result?.result_number || index}
            className="fc-p-3 fc-rounded fc-shadow-sm"
            style={{ backgroundColor: 'var(--fc-bg)' }}
          >
            <a
              href={result.url}
              rel="noreferrer"
              target="_blank"
              className="fc-no-underline fc-text-base fc-font-bold hover:fc-underline"
              style={{ color: 'var(--fc-link)' }}
            >
              {result.title}
            </a>
            <div className="fc-text-sm fc-leading-relaxed" style={{ color: 'var(--fc-text-secondary)' }}>
              {result.description}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
};
