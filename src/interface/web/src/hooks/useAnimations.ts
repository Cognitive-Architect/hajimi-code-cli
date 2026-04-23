import { useEffect, useRef, useState, useCallback } from 'react'

interface UseAnimationOptions {
  duration?: number
  disabled?: boolean
}

const REDUCED_MOTION = '(prefers-reduced-motion: reduce)'

/**
 * useAnimations - accessibility-aware animation hook
 */
export function useAnimations(options: UseAnimationOptions = {}) {
  const { duration = 300, disabled = false } = options
  const [progress, setProgress] = useState(0)
  const rafRef = useRef<number>()
  const startRef = useRef<number>()

  const shouldAnimate = useCallback(() => {
    if (disabled) return false
    return typeof window === 'undefined' || !window.matchMedia(REDUCED_MOTION).matches
  }, [disabled])

  const start = useCallback(() => {
    if (!shouldAnimate()) { setProgress(1); return }
    setProgress(0); startRef.current = undefined
  }, [shouldAnimate])

  useEffect(() => {
    if (progress >= 1) return
    const animate = (t: number) => {
      if (!startRef.current) startRef.current = t
      const p = Math.min(1, (t - startRef.current) / duration)
      setProgress(1 - Math.pow(1 - p, 3))
      if (p < 1) rafRef.current = requestAnimationFrame(animate)
    }
    rafRef.current = requestAnimationFrame(animate)
    return () => { if (rafRef.current) cancelAnimationFrame(rafRef.current) }
  }, [progress, duration])

  return { progress, start, shouldAnimate: shouldAnimate() }
}

/**
 * useThemeTransition - hook for theme transitions
 */
export function useThemeTransition() {
  const [isActive, setIsActive] = useState(false)
  const start = useCallback(() => {
    if (typeof window !== 'undefined' && !window.matchMedia(REDUCED_MOTION).matches) {
      setIsActive(true)
      setTimeout(() => setIsActive(false), 300)
    }
  }, [])
  return { isActive, start, duration: 300 }
}
