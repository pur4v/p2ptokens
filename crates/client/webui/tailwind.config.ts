import type { Config } from 'tailwindcss';

const config: Config = {
  prefix: 'fc-',
  important: '.p2p-chat-root',
  content: ['./src/**/*.{ts,tsx}'],
  theme: {
    extend: {
      colors: {
        'fc-bg': 'var(--fc-bg-color, #ffffff)',
        'fc-bg-card': 'var(--fc-bg-card-color, #f9fafb)',
        'fc-bg-secondary': 'var(--fc-bg-secondary-color, #f3f4f6)',
        'fc-text': 'var(--fc-text-color, #111827)',
        'fc-text-dimmed': 'var(--fc-text-dimmed-color, #6b7280)',
        'fc-text-active': 'var(--fc-text-active-color, #2563eb)',
        'fc-border': 'var(--fc-border-color, #e5e7eb)',
        'fc-border-dimmed': 'var(--fc-border-dimmed-color, #d1d5db)',
        'fc-primary': 'var(--fc-primary-color, #2563eb)',
        'fc-primary-hover': 'var(--fc-primary-hover-color, #1d4ed8)',
        'fc-error': 'var(--fc-error-color, #ef4444)',
        'fc-success': 'var(--fc-success-color, #22c55e)',
      },
    },
  },
  plugins: [],
};

export default config;
