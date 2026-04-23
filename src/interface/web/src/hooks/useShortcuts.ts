import { useEffect, useCallback, useRef } from 'react'

interface ShortcutConfig {
  key: string
  ctrl?: boolean
  alt?: boolean
  shift?: boolean
  handler: () => void
  preventDefault?: boolean
}

interface UseShortcutsOptions {
  shortcuts: ShortcutConfig[]
  enabled?: boolean
}

/**
 * useShortcuts - global keyboard shortcuts management
 * Supports Ctrl+K, Ctrl+/, Escape and custom combinations
 * Prevents browser conflicts with preventDefault
 */
export function useShortcuts({ shortcuts, enabled = true }: UseShortcutsOptions) {
  const shortcutsRef = useRef(shortcuts)
  shortcutsRef.current = shortcuts

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (!enabled) return

    for (const shortcut of shortcutsRef.current) {
      const keyMatch = e.key.toLowerCase() === shortcut.key.toLowerCase()
      const ctrlMatch = !!shortcut.ctrl === e.ctrlKey
      const altMatch = !!shortcut.alt === e.altKey
      const shiftMatch = !!shortcut.shift === e.shiftKey

      if (keyMatch && ctrlMatch && altMatch && shiftMatch) {
        if (shortcut.preventDefault !== false) {
          e.preventDefault()
        }
        shortcut.handler()
        break
      }
    }
  }, [enabled])

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [handleKeyDown])
}

/**
 * Predefined common shortcuts
 */
export const createCommonShortcuts = (actions: {
  focusCommand?: () => void
  clearHistory?: () => void
  toggleTheme?: () => void
  showHelp?: () => void
}): ShortcutConfig[] => [
  { key: 'k', ctrl: true, handler: actions.focusCommand || (() => {}), preventDefault: true },
  { key: 'l', ctrl: true, handler: actions.clearHistory || (() => {}), preventDefault: true },
  { key: 'd', ctrl: true, shift: true, handler: actions.toggleTheme || (() => {}), preventDefault: true },
  { key: '/', ctrl: true, handler: actions.showHelp || (() => {}), preventDefault: true },
  { key: 'Escape', handler: actions.clearHistory || (() => {}), preventDefault: false }
]
