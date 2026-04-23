import { useState, useEffect, useCallback } from 'react'
import { useThemeTransition } from './useAnimations'


type Theme = 'light' | 'dark'

interface UseThemeReturn {
  theme: Theme
  toggleTheme: () => void
  setTheme: (theme: Theme) => void
  isDark: boolean
}

const STORAGE_KEY = 'flexline-theme'

/**
 * useTheme hook - manages theme state with localStorage persistence
 * Supports system preference detection and theme switching
 */
export function useTheme(): UseThemeReturn {
  const [theme, setThemeState] = useState<Theme>(() => {
    if (typeof window === 'undefined') return 'light'
    const stored = localStorage.getItem(STORAGE_KEY) as Theme | null
    if (stored) return stored
    return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  })
  const { startTransition } = useThemeTransition()

  // Apply theme to document element
  useEffect(() => {
    document.documentElement.setAttribute('data-theme', theme)
    localStorage.setItem(STORAGE_KEY, theme)
  }, [theme])

  // Listen to system theme changes
  useEffect(() => {
    const mq = window.matchMedia('(prefers-color-scheme: dark)')
    const handler = (e: MediaQueryListEvent) => {
      if (!localStorage.getItem(STORAGE_KEY)) {
        setThemeState(e.matches ? 'dark' : 'light')
      }
    }
    mq.addEventListener('change', handler)
    return () => mq.removeEventListener('change', handler)
  }, [])

  const setTheme = useCallback((t: Theme) => {
    if (t !== theme) startTransition()
    setThemeState(t)
  }, [theme, startTransition])
  const toggleTheme = useCallback(() => {
    startTransition()
    setThemeState(p => p === 'light' ? 'dark' : 'light')
  }, [startTransition])

  return { theme, toggleTheme, setTheme, isDark: theme === 'dark' }
}
