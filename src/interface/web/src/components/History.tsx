import { useCallback, useRef, useEffect, useState } from 'react'
import { VirtualList } from './VirtualList'
import { debounce } from '../utils/performance'

interface HistoryItem {
  id: string
  command: string
  output?: string
  error?: string
  timestamp: number
}

interface HistoryProps {
  items: HistoryItem[]
  onSelect?: (command: string) => void
  maxHeight?: number
  virtualThreshold?: number
}

/**
 * Escape HTML to prevent XSS attacks
 */
function escapeHtml(text: string): string {
  const div = document.createElement('div')
  div.textContent = text
  return div.innerHTML
}

/**
 * History - displays command history with navigation and XSS protection
 * Supports click to select previous commands and keyboard navigation
 */
const ITEM_HEIGHT = 60 // Estimated height per history item

export function History({ items, onSelect, maxHeight = 400, virtualThreshold = 50 }: HistoryProps) {
  const containerRef = useRef<HTMLDivElement>(null)
  const [newItemIndex, setNewItemIndex] = useState<number | null>(null)

  // Auto-scroll to latest with debounce
  useEffect(() => {
    if (items.length > 0) {
      setNewItemIndex(items.length - 1)
      const timer = setTimeout(() => setNewItemIndex(null), 300)
      return () => clearTimeout(timer)
    }
  }, [items.length])
  
  const scrollToBottom = useCallback(debounce(() => {
    if (containerRef.current && items.length <= virtualThreshold) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight
    }
  }, 100), [items.length, virtualThreshold])

  const handleKeyDown = useCallback((e: React.KeyboardEvent, cmd: string) => {
    if (e.key === 'ArrowUp' || e.key === 'ArrowDown') {
      e.preventDefault()
      onSelect?.(cmd)
    }
  }, [onSelect])

  // Render single history item
  const renderHistoryItem = useCallback((item: HistoryItem, index: number) => (
    <div
      className={`history-item ${item.error ? 'has-error' : ''} ${index === newItemIndex ? 'history-item-enter' : ''}`}
      onClick={() => onSelect?.(item.command)}
      onKeyDown={(e) => handleKeyDown(e, item.command)}
      role="button"
      tabIndex={0}
    >
      <div className="history-command">
        <span className="history-prompt">{'>'}</span>
        <span dangerouslySetInnerHTML={{ __html: escapeHtml(item.command) }} />
      </div>
      {item.output && (
        <pre className="history-output" dangerouslySetInnerHTML={{ __html: escapeHtml(item.output) }} />
      )}
      {item.error && (
        <pre className="history-error" dangerouslySetInnerHTML={{ __html: escapeHtml(item.error) }} />
      )}
    </div>
  ), [onSelect, handleKeyDown, newItemIndex])

  // Use virtual list for large datasets
  const useVirtual = items.length > virtualThreshold

  return (
    <div ref={containerRef} className="history-container" style={{ maxHeight }} tabIndex={0}>
      {items.length === 0 ? (
        <div className="history-empty">No commands yet. Type a command to begin.</div>
      ) : useVirtual ? (
        <VirtualList
          items={items}
          itemHeight={ITEM_HEIGHT}
          containerHeight={maxHeight}
          overscan={3}
          renderItem={renderHistoryItem}
          className="history-virtual-list"
        />
      ) : (
        items.map((item, index) => (
          <div key={item.id}>{renderHistoryItem(item, index)}</div>
        ))
      )}
    </div>
  )
}
