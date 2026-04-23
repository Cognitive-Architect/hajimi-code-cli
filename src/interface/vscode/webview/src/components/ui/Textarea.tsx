import React from 'react';

export interface TextareaProps extends React.TextareaHTMLAttributes<HTMLTextAreaElement> {}

export const Textarea = React.forwardRef<HTMLTextAreaElement, TextareaProps>(
  ({ className = '', ...props }, ref) => {
    return (
      <textarea
        ref={ref}
        className={
          'flex min-h-[60px] w-full rounded-md border border-[var(--vscode-panel-border)] ' +
          'bg-[var(--vscode-input-background)] px-3 py-2 text-sm text-[var(--vscode-input-foreground)] ' +
          'placeholder:text-[var(--vscode-input-placeholderForeground)] ' +
          'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-[var(--vscode-focusBorder)] ' +
          'disabled:cursor-not-allowed disabled:opacity-50 resize-none ' +
          className
        }
        {...props}
      />
    );
  }
);

Textarea.displayName = 'Textarea';
