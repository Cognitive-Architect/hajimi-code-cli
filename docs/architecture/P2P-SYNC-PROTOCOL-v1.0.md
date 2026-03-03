# P2P Sync Protocol v1.0

## 1. Protocol Overview

Hajimi P2P Sync enables **pure local peer-to-peer synchronization** for the "two-laptop scenario" - purely local, no external servers.

**Principles:** Offline-first (local source of truth), end-to-end encryption via sharedSecret (scryptSync), CRDT conflict resolution, `.hctx` compatibility.

## 2. Sync Model (Bidirectional Push/Pull)

```
Device A (.hctx)  ◄════════════════════►  Device B (.hctx)
         WebRTC DataChannel (encrypted + chunked)
```

- `push(peerId)`: Upload local changes
- `pull(peerId)`: Download peer changes  
- `sync(peerId)`: Bidirectional merge (default)

**Chunk Exchange:** 64KB chunks via DataChannelManager, BLAKE3 addressing, hdiff binary diff.

## 3. Conflict Resolution

**Strategy: Last-Write-Wins + Vector Clocks**

```typescript
onConflict(local: Chunk, remote: Chunk): MergeResult {
  // Priority: 1) User choice 2) CRDT auto-merge 3) Timestamp tiebreaker
}
```

**Merge Priority:**
1. Explicit user choice (interactive)
2. CRDT automatic merge (text content)
3. Timestamp + deterministic tiebreaker

## 4. Offline-First Strategy

- All writes → local `.hctx` first
- Async sync, non-blocking
- Offline queue when peer unavailable

```typescript
offlineQueue: Operation[] = [];
flushQueue(): Promise<void>; // Replay when online
```

## 5. NAT Traversal & Discovery

**STUN → TURN → mDNS Fallback:**

```javascript
{
  iceServers: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'turn:optional.turn.server', credential: '...' }
  ],
  discovery: 'mdns' | 'signaling' | 'manual'
}
```

**Discovery:**
- **mDNS**: Zero-config LAN (preferred)
- **Signaling**: Optional WAN coordination
- **Manual**: Explicit peerId entry

## 6. Security Considerations

**Key Derivation (Inherited from datachannel-manager.js):**
```javascript
deriveKey(sharedSecret) {
  return crypto.scryptSync(sharedSecret, 'hajimi-salt-v1', 32, { N: 16384 });
}
```

**Encryption:** AES-256-GCM, per-message IV + authTag. Pairwise trust via out-of-band sharedSecret.

## 7. LCR Integration (.hctx Reuse)

```
P2P Sync Engine → ChunkStorage (.hctx)
             ↓
DataChannelManager (WebRTC, deriveKey, AES-GCM, 64KB chunks)
```

**Sync Protocol:**
1. Exchange chunk manifests (simhash list)
2. Compare via bloom filters
3. Request missing chunks via DataChannel
4. Validate & write to ChunkStorage
5. Update vector clocks

**Two-Laptop Scenario:**
```bash
hajimi sync --peer local  # mDNS discovery
hajimi sync --peer-id <id> --secret <secret>  # Manual
```
