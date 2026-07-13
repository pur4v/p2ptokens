import { useMemo } from 'react';
import { ChatThemeConfig } from '@/types';

export function useThemeVars(themeConfig?: ChatThemeConfig): React.CSSProperties {
  return useMemo(
    () =>
      ({
        '--fc-primary': themeConfig?.primaryColor || '#7c3aed',
        '--fc-primary-hover': themeConfig?.primaryHoverColor || '#6d28d9',
        '--fc-user-bubble': themeConfig?.userBubbleColor || '#f3f4f6',
        '--fc-user-bubble-border': themeConfig?.userBubbleBorder || '#e5e7eb',
        '--fc-assistant-border': themeConfig?.assistantBorderColor || '#e5e7eb',
        '--fc-bg': themeConfig?.backgroundColor || '#ffffff',
        '--fc-text': themeConfig?.textColor || '#111827',
        '--fc-text-secondary': themeConfig?.secondaryTextColor || '#6b7280',
        '--fc-spacing-msg-x': themeConfig?.messagePaddingX || '1rem',
        '--fc-spacing-msg-y': themeConfig?.messagePaddingY || '0.75rem',
        '--fc-spacing-msg-gap': themeConfig?.messageGap || '0.5rem',
        '--fc-spacing-container': themeConfig?.containerPadding || '1rem',
        '--fc-border': themeConfig?.borderColor || '#e5e7eb',
        '--fc-surface': themeConfig?.surfaceColor || '#f9fafb',
        '--fc-surface-hover': themeConfig?.surfaceHoverColor || '#f3f4f6',
        '--fc-success': themeConfig?.successColor || '#22c55e',
        '--fc-error': themeConfig?.errorColor || '#ef4444',
        '--fc-warning': themeConfig?.warningColor || '#eab308',
        '--fc-link': themeConfig?.linkColor || '#3b82f6',
        '--fc-code-bg': themeConfig?.codeBackground || '#f9fafb',
        '--fc-font-family': themeConfig?.fontFamily || "-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif",
        '--fc-radius': themeConfig?.borderRadius || '0.75rem',
        '--fc-radius-input': themeConfig?.inputBorderRadius || '1rem',
      }) as React.CSSProperties,
    [themeConfig]
  );
}
