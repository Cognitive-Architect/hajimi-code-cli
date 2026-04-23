/**
 * Performance utilities
 */

const rIC = typeof window !== 'undefined' && 'requestIdleCallback' in window
  ? window.requestIdleCallback.bind(window)
  : (cb: IdleRequestCallback) => setTimeout(cb, 1)

const cIC = typeof window !== 'undefined' && 'cancelIdleCallback' in window
  ? window.cancelIdleCallback.bind(window)
  : clearTimeout

export const scheduleIdle = rIC
export const cancelIdle = cIC

export function runWhenIdle<T>(task: () => T, timeout = 2000): Promise<T> {
  return new Promise((resolve, reject) => {
    const id = rIC(() => { try { resolve(task()) } catch (e) { reject(e) } }, { timeout })
    if (typeof window !== 'undefined') {
      window.addEventListener('beforeunload', () => cIC(id), { once: true })
    }
  })
}

export function throttle<T extends (...args: unknown[]) => void>(fn: T, limit: number) {
  let th = false
  return (...args: Parameters<T>) => {
    if (!th) { fn(...args); th = true; setTimeout(() => th = false, limit) }
  }
}

export function debounce<T extends (...args: unknown[]) => void>(fn: T, wait: number) {
  let id: ReturnType<typeof setTimeout>
  return (...args: Parameters<T>) => { clearTimeout(id); id = setTimeout(() => fn(...args), wait) }
}
