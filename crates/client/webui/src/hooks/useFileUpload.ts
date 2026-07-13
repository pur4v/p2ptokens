import { useCallback, useRef, useState } from 'react';
import { useAdapter } from '@/provider/ChatProvider';
import { UploadedImage } from '@/types';

export const ALLOWED_IMAGE_TYPES = [
  'image/jpeg',
  'image/png',
  'image/jpg',
  'image/webp',
] as const;

export interface FileUploadItem {
  id: string;
  file: File;
  filename: string;
  url?: string;
  isUploading: boolean;
  error?: string;
}

export interface UseFileUploadOptions {
  maxFiles?: number;
  allowedTypes?: string[];
}

export interface UseFileUploadReturn {
  files: FileUploadItem[];
  uploadFiles: (files: File[]) => Promise<void>;
  removeFile: (id: string) => void;
  retryFile: (id: string) => Promise<void>;
  handleFileInput: (e: React.ChangeEvent<HTMLInputElement>) => void;
  handlePaste: (e: React.ClipboardEvent<HTMLTextAreaElement>) => void;
  isUploading: boolean;
  hasErrors: boolean;
  getUploadedImages: () => UploadedImage[];
  clearAll: () => void;
}

export const useFileUpload = ({
  maxFiles = 5,
  allowedTypes = ALLOWED_IMAGE_TYPES as unknown as string[],
}: UseFileUploadOptions = {}): UseFileUploadReturn => {
  const [files, setFiles] = useState<FileUploadItem[]>([]);
  const adapter = useAdapter();
  const counterRef = useRef(0);

  const generateId = useCallback(() => {
    counterRef.current += 1;
    return `upload_${Date.now()}_${counterRef.current}`;
  }, []);

  const uploadSingleFile = useCallback(
    async (item: FileUploadItem) => {
      try {
        const result = await adapter.uploadFile!(item.file);
        setFiles((prev) =>
          prev.map((f) =>
            f.id === item.id
              ? { ...f, isUploading: false, url: result.url, filename: result.filename }
              : f
          )
        );
      } catch (err: any) {
        setFiles((prev) =>
          prev.map((f) =>
            f.id === item.id
              ? { ...f, isUploading: false, error: err?.message || 'Upload failed' }
              : f
          )
        );
      }
    },
    [adapter]
  );

  const uploadFiles = useCallback(
    async (inputFiles: File[]) => {
      if (!adapter.uploadFile) {
        console.warn('uploadFile not available in adapter');
        return;
      }

      const currentCount = files.length;
      if (currentCount >= maxFiles) return;

      const validFiles = inputFiles.filter((f) => allowedTypes.includes(f.type));
      if (validFiles.length === 0) return;

      const filesToAdd = validFiles.slice(0, maxFiles - currentCount);
      const newItems: FileUploadItem[] = filesToAdd.map((file) => ({
        id: generateId(),
        file,
        filename: file.name,
        isUploading: true,
      }));

      setFiles((prev) => [...prev, ...newItems]);

      await Promise.all(newItems.map(uploadSingleFile));
    },
    [files.length, maxFiles, allowedTypes, generateId, uploadSingleFile, adapter]
  );

  const removeFile = useCallback((id: string) => {
    setFiles((prev) => prev.filter((f) => f.id !== id));
  }, []);

  const retryFile = useCallback(
    async (id: string) => {
      const file = files.find((f) => f.id === id);
      if (!file) return;

      setFiles((prev) =>
        prev.map((f) => (f.id === id ? { ...f, isUploading: true, error: undefined } : f))
      );

      await uploadSingleFile({ ...file, isUploading: true, error: undefined });
    },
    [files, uploadSingleFile]
  );

  const handleFileInput = useCallback(
    async (e: React.ChangeEvent<HTMLInputElement>) => {
      const inputFiles = e.target.files;
      if (inputFiles) {
        await uploadFiles(Array.from(inputFiles));
      }
      e.target.value = '';
    },
    [uploadFiles]
  );

  const handlePaste = useCallback(
    async (e: React.ClipboardEvent<HTMLTextAreaElement>) => {
      const items = e.clipboardData.items;
      const pastedFiles = Array.from(items)
        .filter((item) => item.kind === 'file' && allowedTypes.includes(item.type))
        .map((item) => item.getAsFile())
        .filter((f): f is File => f !== null);

      if (pastedFiles.length > 0) {
        e.preventDefault();
        await uploadFiles(pastedFiles);
      }
    },
    [uploadFiles, allowedTypes]
  );

  const getUploadedImages = useCallback((): UploadedImage[] => {
    return files
      .filter((f) => !f.isUploading && !f.error && f.url)
      .map((f) => ({
        type: f.file.type,
        filename: f.filename,
        url: f.url!,
      }));
  }, [files]);

  const clearAll = useCallback(() => {
    setFiles([]);
  }, []);

  return {
    files,
    uploadFiles,
    removeFile,
    retryFile,
    handleFileInput,
    handlePaste,
    isUploading: files.some((f) => f.isUploading),
    hasErrors: files.some((f) => !!f.error),
    getUploadedImages,
    clearAll,
  };
};
