import React, { memo, useCallback, useState } from 'react';
import { Pencil } from 'lucide-react';
import TextareaAutosize from 'react-textarea-autosize';
import { CopyContent } from '@/components/Shared/CopyContent';
import { usePreviewStoreState, useConfig } from '@/provider/ChatProvider';
import { IUserMessage } from '@/types';

interface UserMessageProps {
  message: IUserMessage;
  images?: any[];
  isEditable?: boolean;
  onEditSubmit?: (messageId: string, newContent: string) => void;
}

export const UserMessage: React.FC<UserMessageProps> = memo(({ message, images, isEditable, onEditSubmit }) => {
  const setChatAttachmentPreview = usePreviewStoreState((s) => s.setChatAttachmentPreview);
  const setCurrentChatAttachmentIndex = usePreviewStoreState((s) => s.setCurrentChatAttachmentIndex);
  const config = useConfig();
  const [isHovered, setIsHovered] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState(message.content);

  const saveText = config.textConfig?.saveAndSendText || 'Save & Send';
  const cancelText = config.textConfig?.cancelText || 'Cancel';

  const handleImageClick = (clickedIndex: number) => {
    if (images) {
      setChatAttachmentPreview(images);
      setCurrentChatAttachmentIndex(clickedIndex);
    }
  };

  const handleEditStart = useCallback(() => {
    setEditContent(message.content);
    setIsEditing(true);
  }, [message.content]);

  const handleEditCancel = useCallback(() => {
    setEditContent(message.content);
    setIsEditing(false);
  }, [message.content]);

  const handleEditSave = useCallback(() => {
    const trimmed = editContent.trim();
    if (!trimmed || trimmed === message.content) {
      handleEditCancel();
      return;
    }
    onEditSubmit?.(message.id, trimmed);
    setIsEditing(false);
  }, [editContent, message.content, message.id, onEditSubmit, handleEditCancel]);

  const handleEditKeyDown = useCallback(
    (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        handleEditSave();
      }
      if (e.key === 'Escape') {
        handleEditCancel();
      }
    },
    [handleEditSave, handleEditCancel]
  );

  return (
    <div className="fc-mb-2 fc-mt-4 fc-ml-auto fc-max-w-[80%]">
      {images && images.length > 0 && (
        <div className="fc-flex fc-flex-wrap fc-gap-2 fc-mb-2 fc-justify-end">
          {images.map((image, index) => (
            <div
              key={index}
              className="fc-relative fc-cursor-pointer"
              onClick={() => handleImageClick(index)}
            >
              <img
                src={image.url}
                alt={`Upload ${index + 1}`}
                className="fc-w-24 fc-h-24 fc-object-cover fc-rounded-lg fc-border"
                style={{ borderColor: 'var(--fc-user-bubble-border)' }}
              />
            </div>
          ))}
        </div>
      )}

      {isEditing ? (
        <div className="fc-flex fc-flex-col fc-gap-2 fc-items-end">
          <div className="fc-w-full fc-border fc-rounded-xl fc-p-2" style={{ borderColor: 'var(--fc-primary)', backgroundColor: 'var(--fc-bg)' }}>
            <TextareaAutosize
              value={editContent}
              onChange={(e) => setEditContent(e.target.value)}
              onKeyDown={handleEditKeyDown}
              minRows={2}
              maxRows={8}
              className="fc-w-full fc-outline-none fc-resize-none fc-bg-transparent fc-text-sm fc-border-0 fc-font-sans"
              style={{ color: 'var(--fc-text)' }}
              autoFocus
            />
          </div>
          <div className="fc-flex fc-gap-2">
            <button
              type="button"
              onClick={handleEditCancel}
              className="fc-px-3 fc-py-1.5 fc-text-xs fc-font-medium fc-rounded-lg fc-border-0 fc-cursor-pointer fc-transition-colors"
              style={{ color: 'var(--fc-text-secondary)', backgroundColor: 'var(--fc-user-bubble)' }}
            >
              {cancelText}
            </button>
            <button
              type="button"
              onClick={handleEditSave}
              disabled={!editContent.trim()}
              className="fc-px-3 fc-py-1.5 fc-text-xs fc-font-medium fc-text-white fc-rounded-lg fc-border-0 fc-cursor-pointer fc-transition-colors disabled:fc-opacity-50 disabled:fc-cursor-not-allowed"
              style={{ backgroundColor: 'var(--fc-primary)' }}
            >
              {saveText}
            </button>
          </div>
        </div>
      ) : (
        <div
          className="fc-w-full fc-flex fc-justify-end fc-relative"
          onMouseEnter={() => setIsHovered(true)}
          onMouseLeave={() => setIsHovered(false)}
        >
          <div
            className="fc-px-4 fc-py-3 fc-w-fit fc-border fc-rounded-[12px_12px_0_12px] fc-font-medium fc-text-sm fc-whitespace-pre-wrap"
            style={{
              backgroundColor: 'var(--fc-user-bubble)',
              borderColor: 'var(--fc-user-bubble-border)',
              color: 'var(--fc-text)',
            }}
          >
            {message.content}
          </div>
          <div className="fc-flex fc-flex-col fc-gap-1">
            <CopyContent content={message.content} isParentHovered={isHovered} />
            {isEditable && isHovered && (
              <button
                type="button"
                onClick={handleEditStart}
                className="fc-p-1 fc-bg-transparent fc-border-0 fc-cursor-pointer fc-transition-colors"
                style={{ color: 'var(--fc-text-secondary)' }}
                title="Edit message"
              >
                <Pencil size={14} />
              </button>
            )}
          </div>
        </div>
      )}
    </div>
  );
});

UserMessage.displayName = 'UserMessage';
