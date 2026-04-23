/** Minimal AbortController declarations for VSCode extension host. */
declare global {
  interface AbortSignal {
    readonly aborted: boolean;
    addEventListener(type: 'abort', listener: () => void): void;
    removeEventListener(type: 'abort', listener: () => void): void;
  }
  interface AbortController { readonly signal: AbortSignal; abort(): void; }
  var AbortController: { new(): AbortController; prototype: AbortController; };
}
export {};
