import { useEffect, useRef, useState } from 'react'

interface StreamOutputProps {
  content: string
  streaming?: boolean
  chunkSize?: number
  delayMs?: number
}

/**
 * StreamOutput - renders text with streaming effect and auto-scroll
 * Supports chunked rendering simulation for terminal-like experience
 */
export function StreamOutput({
  content, streaming = false, chunkSize = 10, delayMs = 50
}: StreamOutputProps) {
  const [displayed, setDisplayed] = useState('')
  const [isComplete, setIsComplete] = useState(false)
  const containerRef = useRef<HTMLDivElement>(null)
  const indexRef = useRef(0)

  // Streaming effect with chunked output
  useEffect(() => {
    if (!streaming) {
      setDisplayed(content)
      setIsComplete(true)
      return
    }
    setIsComplete(false)
    indexRef.current = 0
    setDisplayed('')

    const interval = setInterval(() => {
      const next = indexRef.current + chunkSize
      if (next >= content.length) {
        setDisplayed(content)
        setIsComplete(true)
        clearInterval(interval)
      } else {
        setDisplayed(content.slice(0, next))
        indexRef.current = next
      }
    }, delayMs)

    return () => clearInterval(interval)
  }, [content, streaming, chunkSize, delayMs])

  // Auto-scroll to bottom
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = containerRef.current.scrollHeight
    }
  }, [displayed])

  return (
    <div ref={containerRef} className="stream-output">
      <pre className="stream-content">{displayed}</pre>
      {!isComplete && streaming && <span className="stream-cursor">▋</span>}
    </div>
  )
}
