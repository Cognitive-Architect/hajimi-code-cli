import { useState, useEffect, useRef, useCallback } from 'react'

interface MCPMessage { type: string; payload: unknown; id?: string }

interface UseMCPReturn {
  connected: boolean
  connecting: boolean
  error: Error | null
  send: (msg: MCPMessage) => void
  messages: MCPMessage[]
  connect: () => void
  disconnect: () => void
}

interface UseMCPOptions { url?: string; autoConnect?: boolean; reconnect?: boolean; reconnectInterval?: number }

/**
 * useMCP hook - manages MCP WebSocket communication with cleanup
 * Handles connection lifecycle, message sending, and auto-reconnect
 */
export function useMCP(options: UseMCPOptions = {}): UseMCPReturn {
  const { url = 'ws://localhost:8080/mcp', autoConnect = true, reconnect = true, reconnectInterval = 3000 } = options
  const [connected, setConnected] = useState(false)
  const [connecting, setConnecting] = useState(false)
  const [error, setError] = useState<Error | null>(null)
  const [messages, setMessages] = useState<MCPMessage[]>([])
  const wsRef = useRef<WebSocket | null>(null)
  const timerRef = useRef<ReturnType<typeof setTimeout> | null>(null)

  // Cleanup WebSocket and timers
  const cleanup = useCallback(() => {
    if (timerRef.current) { clearTimeout(timerRef.current); timerRef.current = null }
    if (wsRef.current) { wsRef.current.close(); wsRef.current = null }
  }, [])

  // Connect to WebSocket
  const connect = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) return
    cleanup(); setConnecting(true); setError(null)
    const ws = new WebSocket(url)
    wsRef.current = ws
    ws.onopen = () => { setConnected(true); setConnecting(false) }
    ws.onmessage = (e) => { try { setMessages(p => [...p, JSON.parse(e.data)]) } catch {} }
    ws.onerror = () => { setError(new Error('WebSocket error')); setConnecting(false) }
    ws.onclose = () => { setConnected(false); setConnecting(false); if (reconnect) timerRef.current = setTimeout(connect, reconnectInterval) }
  }, [url, reconnect, reconnectInterval, cleanup])

  const disconnect = useCallback(() => { cleanup(); setConnected(false) }, [cleanup])

  const send = useCallback((msg: MCPMessage) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) wsRef.current.send(JSON.stringify(msg))
    else setError(new Error('Not connected'))
  }, [])

  // Auto-connect on mount, cleanup on unmount
  useEffect(() => { if (autoConnect) connect(); return cleanup }, [autoConnect, connect, cleanup])

  return { connected, connecting, error, send, messages, connect, disconnect }
}
