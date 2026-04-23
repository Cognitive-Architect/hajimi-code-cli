import { ReactNode, useState, useCallback, MouseEvent, useEffect } from 'react'
import { useAnimations } from '../hooks/useAnimations'
import { debounce } from '../utils/performance'

export interface PaneProps {
  children: ReactNode
  title?: string
  width?: number
  minWidth?: number
  maxWidth?: number
  defaultWidth?: number
  resizable?: boolean
  collapsible?: boolean
  collapsed?: boolean
  onResize?: (width: number) => void
  onCollapse?: (collapsed: boolean) => void
}

/**
 * Pane component - resizable panel with title header and collapse support
 * Supports flexible width configuration, resize interactions, and collapse state
 */
export function Pane({
  children, title, width, minWidth = 150, maxWidth = 800, defaultWidth = 250,
  resizable = true, collapsible = false, collapsed = false, onResize, onCollapse
}: PaneProps) {
  const [currentWidth, setCurrentWidth] = useState(defaultWidth)
  const [isResizing, setIsResizing] = useState(false)
  const [isCollapsed, setIsCollapsed] = useState(collapsed)

  // Sync collapsed state with prop changes
  useEffect(() => setIsCollapsed(collapsed), [collapsed])

  // Animation for smooth resize
  const { shouldAnimate } = useAnimations({ duration: 150, disabled: isResizing })
  
  // Debounced resize handler
  const debouncedResize = useCallback(debounce((w: number) => onResize?.(w), 50), [onResize])

  // Handle resize start
  const handleResizeStart = useCallback((e: MouseEvent) => {
    if (!resizable || isCollapsed) return
    e.preventDefault()
    setIsResizing(true)
  }, [resizable, isCollapsed])

  // Handle resize movement
  const handleResizeMove = useCallback((e: globalThis.MouseEvent) => {
    if (!isResizing) return
    const newWidth = Math.max(minWidth, Math.min(maxWidth, e.clientX))
    setCurrentWidth(newWidth)
    debouncedResize(newWidth)
  }, [isResizing, minWidth, maxWidth, onResize])

  // Handle resize end
  const handleResizeEnd = useCallback(() => setIsResizing(false), [])

  // Toggle collapse state
  const toggleCollapse = useCallback(() => {
    if (!collapsible) return
    const newState = !isCollapsed
    setIsCollapsed(newState)
    onCollapse?.(newState)
  }, [collapsible, isCollapsed, onCollapse])

  // Attach global mouse events during resize
  useEffect(() => {
    if (!isResizing) return
    window.addEventListener('mousemove', handleResizeMove)
    window.addEventListener('mouseup', handleResizeEnd)
    return () => {
      window.removeEventListener('mousemove', handleResizeMove)
      window.removeEventListener('mouseup', handleResizeEnd)
    }
  }, [isResizing, handleResizeMove, handleResizeEnd])

  const displayWidth = isCollapsed ? minWidth : (width ?? currentWidth)
  const isFlexible = width === undefined && !resizable && !isCollapsed

  return (
    <div className={`pane ${isResizing ? 'pane-resizing' : ''} ${shouldAnimate ? 'pane-smooth-resize' : ''}`} style={{
      flex: isFlexible ? 1 : undefined, width: isFlexible ? undefined : displayWidth,
      minWidth: isCollapsed ? undefined : minWidth, maxWidth, display: 'flex',
      flexDirection: 'column', overflow: 'hidden', position: 'relative'
    }}>
      {title && (
        <div className="pane-header" onClick={toggleCollapse} style={{ cursor: collapsible ? 'pointer' : 'default' }}>
          <span className="pane-title">{title}</span>
          {collapsible && <span className="pane-collapse-indicator">{isCollapsed ? '▶' : '▼'}</span>}
        </div>
      )}
      <div className="pane-content" style={{ display: isCollapsed ? 'none' : 'block' }}>{children}</div>
      {resizable && !isCollapsed && (
        <div className={`pane-resizer ${isResizing ? 'resizing' : ''}`} onMouseDown={handleResizeStart} role="separator" aria-label="Resize panel" />
      )}
    </div>
  )
}
