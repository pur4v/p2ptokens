import React, { useMemo } from 'react';
import { LoaderCircle, RefreshCw, X } from 'lucide-react';
import { FileUploadItem } from '@/hooks/useFileUpload';

interface FilePreviewProps {
  files: FileUploadItem[];
  onRemove: (id: string) => void;
  onRetry: (id: string) => void;
}

export const FilePreview: React.FC<FilePreviewProps> = ({ files, onRemove, onRetry }) => {
  if (files.length === 0) return null;

  return (
    <div className="fc-flex fc-flex-wrap fc-gap-2 fc-items-end fc-px-3 fc-pt-2">
      {files.map((item) => (
        <FilePreviewItem
          key={item.id}
          item={item}
          onRemove={onRemove}
          onRetry={onRetry}
        />
      ))}
    </div>
  );
};

const FilePreviewItem: React.FC<{
  item: FileUploadItem;
  onRemove: (id: string) => void;
  onRetry: (id: string) => void;
}> = ({ item, onRemove, onRetry }) => {
  const previewUrl = useMemo(() => URL.createObjectURL(item.file), [item.file]);

  return (
    <div className="fc-group fc-relative fc-p-1 fc-border fc-rounded-md" style={{ borderColor: 'var(--fc-border)' }}>
      {/* Loading overlay */}
      {item.isUploading && (
        <div className="fc-absolute fc-inset-0 fc-bg-black/50 fc-rounded-md fc-flex fc-items-center fc-justify-center fc-z-10">
          <LoaderCircle size={20} className="fc-animate-spin fc-text-white" />
        </div>
      )}

      {/* Error overlay */}
      {item.error && (
        <div className="fc-absolute fc-inset-0 fc-bg-black/50 fc-rounded-md fc-flex fc-items-center fc-justify-center fc-z-10">
          <button
            type="button"
            onClick={(e) => {
              e.stopPropagation();
              onRetry(item.id);
            }}
            className="fc-text-white fc-bg-transparent fc-border-0 fc-cursor-pointer fc-p-0"
          >
            <RefreshCw size={20} />
          </button>
        </div>
      )}

      <img
        src={previewUrl}
        alt={item.filename}
        className="fc-h-10 fc-w-10 fc-object-cover fc-rounded"
      />

      {/* Remove button */}
      <button
        type="button"
        onClick={(e) => {
          e.stopPropagation();
          onRemove(item.id);
        }}
        className="fc-absolute fc--top-2 fc--right-2 fc-rounded-full fc-w-4 fc-h-4 fc-flex fc-items-center fc-justify-center fc-text-white fc-z-20 fc-opacity-0 group-hover:fc-opacity-100 fc-transition-opacity fc-border-0 fc-cursor-pointer fc-p-0"
        style={{ backgroundColor: 'var(--fc-error)' }}
      >
        <X size={10} />
      </button>
    </div>
  );
};
