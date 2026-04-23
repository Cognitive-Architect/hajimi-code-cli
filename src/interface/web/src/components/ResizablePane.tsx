import { ReactNode, useState, useCallback, useRef, useEffect } from 'react'

interface ResizablePaneProps {
  children: ReactNode
  direction: 'horizontal' | 'vertical'
  defaultSize?: number
  minSize?: number
  maxSize?: number
  onResize?: (size: number) => void
}

/**
 * ResizablePane - resizable container with horizontal/vertical support
 */
export function ResizablePane({
  children, direction, defaultSize = 250, minSize = 100, maxSize = 600, onResize
}: ResizablePaneProps) {
  const [size, setSize] = useState(defaultSize)
  const [isResizing, setIsResizing] = useState(false)
  const startPos = useRef(0)
  const startSize = useRef(0)

  const onMove = useCallback((e: MouseEvent) => {
    const delta = direction === 'horizontal' ? e.clientX - startPos.current : e.clientY - startPos.current
    const newSize = Math.max(minSize, Math.min(maxSize, startSize.current + delta))
    setSize(newSize)
    onResize?.(newSize)
  }, [direction, minSize, maxSize, onResize])

  const onUp = useCallback(() => {
    setIsResizing(false)
    document.body.style.cursor = ''
    document.body.style.userSelect = ''
  }, [])

  const onDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault()
    setIsResizing(true)
    startPos.current = direction === 'horizontal' ? e.clientX : e.clientY
    startSize.current = size
  }, [direction, size])

  useEffect(() => {
    if (!isResizing) return
    window.addEventListener('mousemove', onMove)
    window.addEventListener('mouseup', onUp)
    document.body.style.cursor = direction === 'horizontal' ? 'col-resize' : 'row-resize'
    document.body.style.userSelect = 'none'
    return () => {
      window.removeEventListener('mousemove', onMove)
      window.removeEventListener('mouseup', onUp)
    }
  }, [isResizing, onMove, onUp, direction])

  const isH = direction === 'horizontal'
  return (
    <div className={`resizable-pane ${isResizing ? 'resizing' : ''}`} style={{
      position: 'relative', [isH ? 'width' : 'height']: size,
      [isH ? 'minWidth' : 'minHeight']: minSize, [isH ? 'maxWidth' : 'maxHeight']: maxSize,
      display: 'flex', flexDirection: isH ? 'column' : 'row', overflow: 'hidden'
    }}>
      {children}
      <div onMouseDown={onDown} role="separator" style={{
        position: 'absolute', [isH ? 'right' : 'bottom']: 0, [isH ? 'top' : 'left']: 0,
        [isH ? 'width' : 'height']: '4px', [isH ? 'height' : 'width']: '100%',
        cursor: isH ? 'col-resize' : 'row-resize', zIndex: 10
      }} />
    </div>
  )
}
