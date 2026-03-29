/**
 * SyncEngine - TypeScript Interface Definitions (≤200 lines)
 * Sprint 5: P2P Synchronization Architecture
 */

import { EventEmitter } from 'events';

// Core Types
type Simhash = bigint;
type PeerId = string;

interface VectorClock { [nodeId: string]: number; }

type OperationType = 'CREATE' | 'UPDATE' | 'DELETE';

interface Operation {
  id: string;
  type: OperationType;
  targetChunk: Simhash;
  timestamp: number;
  vectorClock: VectorClock;
}

interface Chunk {
  simhash: Simhash;
  data: Buffer;
  metadata: {
    ctime: number; mtime: number; size: number; hash: string;
    crdtType?: 'yjs' | 'automerge' | 'raw';
  };
  vectorClock: VectorClock;
}

interface MergeResult {
  winner: 'local' | 'remote' | 'merged';
  mergedData?: Buffer;
  conflictResolved: boolean;
  resolutionStrategy: 'crdt' | 'timestamp' | 'manual';
}

interface SyncResult {
  success: boolean;
  pushed: number; pulled: number;
  conflicts: number; conflictsResolved: number;
  duration: number; error?: string;
}

interface PeerState {
  peerId: PeerId; connected: boolean;
  lastSeen: number; vectorClock: VectorClock;
  capabilities: string[];
}

// Bidirectional Sync Engine Interface
export interface SyncEngine extends EventEmitter {
  // Connection management
  connect(peerId: PeerId, sharedSecret: string): Promise<void>;
  disconnect(peerId: PeerId): void;
  isConnected(peerId: PeerId): boolean;
  getPeerState(peerId: PeerId): PeerState | null;
  
  // Bidirectional synchronization (push/pull/sync)
  sync(peerId: PeerId, sharedSecret: string): Promise<SyncResult>;
  push(peerId: PeerId): Promise<void>;
  pull(peerId: PeerId): Promise<void>;
  
  // Offline support
  offlineQueue: Operation[];
  queueOperation(op: Operation): void;
  flushQueue(): Promise<void>;
  clearQueue(): void;
  
  // Conflict resolution callback
  onConflict: (local: Chunk, remote: Chunk) => MergeResult | Promise<MergeResult>;
  
  // Progress callbacks
  onProgress?: (peerId: PeerId, percent: number, direction: 'push' | 'pull') => void;
  onChunkSynced?: (peerId: PeerId, chunk: Chunk, direction: 'push' | 'pull') => void;
  
  // Discovery & lifecycle
  discoverPeers(): Promise<PeerId[]>;
  start(): Promise<void>;
  stop(): Promise<void>;
}

// CRDT Handler (Pluggable)
export interface CRDTHandler {
  readonly type: 'yjs' | 'automerge' | 'custom';
  merge(local: Buffer, remote: Buffer): Buffer;
  getStateVector(data: Buffer): Buffer;
  applyUpdate(doc: Buffer, update: Buffer): Buffer;
  validate(data: Buffer): boolean;
}

// Discovery Provider
export interface DiscoveryProvider extends EventEmitter {
  readonly method: 'mdns' | 'signaling' | 'manual';
  start(): Promise<void>;
  stop(): Promise<void>;
  findPeers(): Promise<Array<{ peerId: PeerId; address: string; port: number }>>;
  advertise(peerId: PeerId, port: number): Promise<void>;
}

// Storage Adapter (LCR Integration)
export interface StorageAdapter {
  readChunk(simhash: Simhash): Promise<Chunk | null>;
  writeChunk(chunk: Chunk): Promise<void>;
  hasChunk(simhash: Simhash): Promise<boolean>;
  getManifest(): Promise<Simhash[]>;
  getChangesSince(vc: VectorClock): Promise<Chunk[]>;
}

// Configuration
export interface SyncConfig {
  discovery: 'mdns' | 'signaling' | 'manual';
  signalingUrl?: string;
  iceServers: RTCIceServer[];
  crdt: 'yjs' | 'automerge';
  conflictStrategy: 'crdt-auto' | 'timestamp' | 'manual';
  syncInterval: number; retryInterval: number;
  maxQueueSize: number; maxConcurrentSyncs: number;
}
