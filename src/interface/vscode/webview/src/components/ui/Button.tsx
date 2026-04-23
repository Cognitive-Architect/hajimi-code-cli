import React from 'react';

export interface ButtonProps extends React.ButtonHTMLAttributes<HTMLButtonElement> {
  variant?: 'default' | 'destructive' | 'outline' | 'ghost';
  size?: 'default' | 'sm' | 'lg' | 'icon';
}

export const Button = React.forwardRef<HTMLButtonElement, ButtonProps>(
  ({ className = '', variant = 'default', size = 'default', ...props }, ref) => {
    const base =
      'inline-flex items-center justify-center rounded-md font-medium transition-colors ' +
      'focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring ' +
      'disabled:pointer-events-none disabled:opacity-50';

    const variants: Record<string, string> = {
      default:
        'bg-[var(--vscode-button-background)] text-[var(--vscode-button-foreground)] shadow hover:opacity-90',
      destructive: 'bg-red-600 text-white shadow-sm hover:bg-red-700',
      outline:
        'border border-[var(--vscode-panel-border)] bg-transparent shadow-sm hover:bg-[var(--vscode-list-hoverBackground)] hover:text-[var(--vscode-list-hoverForeground)]',
      ghost: 'hover:bg-[var(--vscode-list-hoverBackground)] hover:text-[var(--vscode-list-hoverForeground)]',
    };

    const sizes: Record<string, string> = {
      default: 'h-9 px-4 py-2 text-sm',
      sm: 'h-8 rounded-md px-3 text-xs',
      lg: 'h-10 rounded-md px-8 text-base',
      icon: 'h-9 w-9',
    };

    return (
      <button
        ref={ref}
        className={`${base} ${variants[variant] ?? variants.default} ${sizes[size] ?? sizes.default} ${className}`}
        {...props}
      />
    );
  }
);

Button.displayName = 'Button';
