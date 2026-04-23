import { useState, useRef, useCallback, KeyboardEvent, FormEvent } from 'react'

interface CommandInputProps {
  onSubmit: (command: string) => void
  onCancel?: () => void
  placeholder?: string
  disabled?: boolean
  history?: string[]
}

/**
 * CommandInput - terminal-like command input with history navigation
 * Supports Enter to submit, Ctrl+C to cancel, and up/down history
 */
export function CommandInput({
  onSubmit, onCancel, placeholder = 'Enter command...', disabled = false, history = []
}: CommandInputProps) {
  const [input, setInput] = useState('')
  const [historyIndex, setHistoryIndex] = useState(-1)
  const inputRef = useRef<HTMLInputElement>(null)

  const handleSubmit = useCallback((e: FormEvent) => {
    e.preventDefault()
    const trimmed = input.trim()
    if (trimmed && !disabled) {
      onSubmit(trimmed)
      setInput('')
      setHistoryIndex(-1)
    }
  }, [input, disabled, onSubmit])

  const handleKeyDown = useCallback((e: KeyboardEvent<HTMLInputElement>) => {
    // Ctrl+C to cancel
    if (e.ctrlKey && e.key === 'c') {
      e.preventDefault()
      onCancel?.()
      return
    }

    // History navigation with ArrowUp/ArrowDown
    if (e.key === 'ArrowUp') {
      e.preventDefault()
      if (historyIndex < history.length - 1) {
        const newIndex = historyIndex + 1
        setHistoryIndex(newIndex)
        setInput(history[history.length - 1 - newIndex] || '')
      }
    } else if (e.key === 'ArrowDown') {
      e.preventDefault()
      if (historyIndex > 0) {
        const newIndex = historyIndex - 1
        setHistoryIndex(newIndex)
        setInput(history[history.length - 1 - newIndex] || '')
      } else if (historyIndex === 0) {
        setHistoryIndex(-1)
        setInput('')
      }
    }
  }, [history, historyIndex, onCancel])

  return (
    <form onSubmit={handleSubmit} className="command-input">
      <span className="command-prompt">{disabled ? '…' : '>'}</span>
      <input
        ref={inputRef}
        type="text"
        value={input}
        onChange={(e) => setInput(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={placeholder}
        disabled={disabled}
        className="command-field"
        autoFocus
      />
    </form>
  )
}
