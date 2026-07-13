import React, { useRef } from 'react';
import { Paperclip } from 'lucide-react';
import { ALLOWED_IMAGE_TYPES } from '@/hooks/useFileUpload';

interface FileUploadButtonProps {
  onFilesSelected: (files: File[]) => void;
  disabled?: boolean;
  accept?: string;
}

export const FileUploadButton: React.FC<FileUploadButtonProps> = ({
  onFilesSelected,
  disabled = false,
  accept = (ALLOWED_IMAGE_TYPES as readonly string[]).join(','),
}) => {
  const inputRef = useRef<HTMLInputElement>(null);

  const handleClick = () => {
    if (!disabled) {
      inputRef.current?.click();
    }
  };

  const handleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = e.target.files;
    if (files) {
      onFilesSelected(Array.from(files));
    }
    e.target.value = '';
  };

  return (
    <>
      <button
        type="button"
        className={`fc-bg-transparent fc-border-0 fc-p-1 ${
          disabled ? 'fc-cursor-not-allowed fc-opacity-40' : 'fc-cursor-pointer'
        }`}
        style={{ color: 'var(--fc-text-secondary)' }}
        onClick={handleClick}
        disabled={disabled}
        title="Attach file"
      >
        <Paperclip size={18} />
      </button>
      <input
        ref={inputRef}
        type="file"
        className="fc-hidden"
        accept={accept}
        onChange={handleChange}
        disabled={disabled}
        multiple
      />
    </>
  );
};
