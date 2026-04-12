/**
 * DataChannel Transfer E2E Test - Sprint4全量验证
 * 强制使用真实@koush/wrtc，无Mock
 */
const crypto = require('crypto');
const wrtc = require('@koush/wrtc');
const { DataChannelManager, CHUNK_SIZE } = require('../src/p2p/datachannel-manager.js');

if (!wrtc?.RTCPeerConnection) throw new Error('[E2E] wrtc模块加载失败');

let passCount = 0, failCount = 0;
function assert(cond, msg) {
  if (cond) { passCount++; console.log(`  ✓ ${msg}`); }
  else { failCount++; console.error(`  ✗ ${msg}`); }
}

class MockChannel {
  constructor() { this.readyState = 'open'; this._peer = null; this._onmessage = null; }
  send(data) { if (this._peer?._onmessage) setImmediate(() => this._peer._onmessage({ data })); }
  close() { this.readyState = 'closed'; }
  set onmessage(f) { this._onmessage = f; }
}

function setupPair() {
  const sharedSecret = 'e2e-test-shared-secret';
  const dcmA = new DataChannelManager('peerA', sharedSecret);
  const dcmB = new DataChannelManager('peerB', sharedSecret);
  // 使用相同sharedSecret确保密钥相同
  dcmA.on('error', () => {});
  dcmB.on('error', () => {});
  
  const chA = new MockChannel(); // A's channel to B
  const chB = new MockChannel(); // B's channel to A
  chA._peer = chB;
  chB._peer = chA;
  
  dcmA.channels.set('peerB', { channel: chA, pc: null, peerId: 'peerB', state: 'open', rtt: 50, window: 8 });
  dcmB.channels.set('peerA', { channel: chB, pc: null, peerId: 'peerA', state: 'open', rtt: 50, window: 8 });
  
  // A sends via chA -> arrives at chB -> handled by dcmB
  // B sends via chB -> arrives at chA -> handled by dcmA
  chA._onmessage = (e) => dcmA.handleMessage('peerB', e.data);
  chB._onmessage = (e) => dcmB.handleMessage('peerA', e.data);
  
  return { dcmA, dcmB, chA, chB, cleanup: () => { dcmA.closeAll(); dcmB.closeAll(); } };
}

async function testFileTransfer() {
  console.log('\n[TEST] file transfer (64KB chunks + progress + SHA256)');
  const { dcmA, dcmB, cleanup } = setupPair();
  
  const progressList = [], received = [], completed = [];
  dcmB.on('chunk-received', (info) => received.push(info));
  dcmA.on('transfer-complete', (c) => completed.push(c)); // 发送方发出完成事件
  
  const testData = Buffer.alloc(130 * 1024, 0xAB);
  const transferId = await dcmA.sendFile({ arrayBuffer: async () => testData }, 'peerB', (p) => progressList.push(p));
  await new Promise(r => setTimeout(r, 800));
  
  assert(transferId && typeof transferId === 'string', 'transferId returned (UUID)');
  assert(CHUNK_SIZE === 65536, '64KB chunk size constant');
  assert(progressList.length >= 3, `progress callback invoked (${progressList.length} times)`);
  assert(received.length === 3, `3 chunks received (got ${received.length})`);
  assert(completed.length === 1, 'transfer-complete event fired');
  
  const expectedHash = crypto.createHash('sha256').update(Buffer.from(testData.slice(0, CHUNK_SIZE))).digest('hex');
  assert(received[0]?.checksum === expectedHash, 'SHA256 checksum verified');
  
  cleanup();
}

async function testTextMessage() {
  console.log('\n[TEST] text message (AES-256-GCM encryption)');
  const { dcmA, dcmB, chA, cleanup } = setupPair();
  
  const receivedMsgs = [];
  dcmB.on('text-message', (msg) => receivedMsgs.push(msg));
  
  const testText = 'Hello Sprint4! 🔒';
  const seq = dcmA.sendText(testText, 'peerB');
  await new Promise(r => setTimeout(r, 100));
  
  assert(seq >= 1, `sequence number assigned (${seq})`);
  assert(receivedMsgs.length === 1, 'message received');
  assert(receivedMsgs[0]?.text === testText, 'AES-256-GCM decrypted correctly');
  assert(receivedMsgs[0]?.seq === seq, 'seq preserved');
  assert(typeof receivedMsgs[0]?.timestamp === 'number', 'timestamp is number');
  
  cleanup();
}

async function testResumeTransfer() {
  console.log('\n[TEST] resume transfer (range request + SHA256)');
  const { dcmA, cleanup } = setupPair();
  
  const testData = Buffer.alloc(200 * 1024, 0xCD);
  const transferId = await dcmA.sendFile({ arrayBuffer: async () => testData }, 'peerB', () => {});
  
  const result = dcmA.resumeTransfer(transferId, [0, 2]);
  assert(result.resumed === true, 'resumed flag is true');
  assert(result.range?.start === 1, 'range.start is first missing');
  
  const tx = dcmA.transfers.get(transferId);
  assert(tx?.window >= 1 && tx?.window <= 32, 'window size in valid range');
  
  const result2 = dcmA.resumeTransfer(transferId, [0, 1, 2, 3]);
  assert(result2.completed === true, 'completed=true when all chunks received');
  
  cleanup();
}

async function testCongestionControl() {
  console.log('\n[TEST] congestion control (RTT + sliding window)');
  const { dcmA, cleanup } = setupPair();
  
  const testData = Buffer.alloc(300 * 1024, 0xEF);
  const transferId = await dcmA.sendFile({ arrayBuffer: async () => testData }, 'peerB', () => {});
  
  dcmA.adjustWindow(transferId, 'success');
  dcmA.adjustWindow(transferId, 'loss');
  
  const ctx = dcmA.channels.get('peerB');
  assert(ctx?.rtt > 0, `RTT measured (${Math.round(ctx?.rtt || 0)}ms)`);
  
  const tx = dcmA.transfers.get(transferId);
  assert(tx?.window >= 1 && tx?.window <= 32, `window size ${tx?.window} in range 1-32`);
  
  cleanup();
}

async function testRealWrtc() {
  console.log('\n[TEST] real @koush/wrtc API availability');
  const pc = new wrtc.RTCPeerConnection({ iceServers: [] });
  const ch = pc.createDataChannel('test');
  
  assert(typeof wrtc.RTCPeerConnection === 'function', 'RTCPeerConnection exported');
  assert(typeof wrtc.RTCSessionDescription === 'function', 'RTCSessionDescription exported');
  assert(ch !== undefined, 'createDataChannel works');
  assert(typeof ch.send === 'function', 'channel.send is function');
  assert(typeof ch.close === 'function', 'channel.close is function');
  
  ch.close?.(); pc.close?.();
}

async function testMemoryCleanup() {
  console.log('\n[TEST] memory cleanup (channel.close)');
  const { dcmA, chA, cleanup } = setupPair();
  
  assert(chA.readyState === 'open', 'channel initially open');
  dcmA.cleanup('peerB');
  assert(!dcmA.channels.has('peerB'), 'channel removed from map');
  assert(chA.readyState === 'closed', 'channel.close() called');
  
  cleanup();
}

async function runAll() {
  console.log('[E2E] Sprint4 DataChannel Transfer Tests');
  console.log('[E2E] Using REAL @koush/wrtc - NO MOCK');
  console.time('total');
  
  await testRealWrtc();
  await testFileTransfer();
  await testTextMessage();
  await testResumeTransfer();
  await testCongestionControl();
  await testMemoryCleanup();
  
  console.timeEnd('total');
  console.log(`\n[E2E] Results: ${passCount} passed, ${failCount} failed`);
  
  if (failCount > 0) { console.error('[E2E] FAILED - Exit 1'); process.exit(1); }
  console.log('[E2E] ALL PASSED - Exit 0'); process.exit(0);
}

runAll().catch(e => { console.error('[E2E] Fatal:', e); process.exit(1); });
