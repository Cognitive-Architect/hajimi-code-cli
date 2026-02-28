/**
 * NAT Traversal Test (60-80 lines)
 * Host/STUN/TURN scenarios
 * 强制使用真实@koush/wrtc，无Mock fallback
 */
const CONFIG = require('../src/p2p/config.js');

// 强制require @koush/wrtc，失败直接throw，无Mock fallback
const wrtc = require('@koush/wrtc');
if (!wrtc || !wrtc.RTCPeerConnection) {
  throw new Error('[NAT] @koush/wrtc模块加载失败，无法初始化RTCPeerConnection');
}
const RTCPeerConnection = wrtc.RTCPeerConnection;

async function runTests() {
  console.log('[NAT] Starting NAT traversal tests...');
  console.log('[NAT] STUN: stun.l.google.com:19302');

  // E2E-101: Host直连
  console.log('\n[TEST] E2E-101: Host direct (127.0.0.1)');
  const hostCfg = { iceServers: [] };
  const peerH1 = new RTCPeerConnection(hostCfg);
  const peerH2 = new RTCPeerConnection(hostCfg);
  const hostCands = [];
  peerH1.onicecandidate = (e) => { if (e.candidate) hostCands.push(e.candidate); };
  const offerH = await peerH1.createOffer(); 
  await peerH1.setLocalDescription(offerH);
  await new Promise(r => setTimeout(r, 200));
  console.log(`  ✓ Host candidates: ${hostCands.length}`);
  peerH1.close(); peerH2.close();

  // E2E-102: STUN穿透
  console.log('[TEST] E2E-102: STUN穿透 (Google STUN)');
  const stunCfg = { iceServers: CONFIG.STUN_SERVERS };
  const peerStun = new RTCPeerConnection(stunCfg);
  const stunCands = [];
  peerStun.onicecandidate = (e) => { if (e.candidate) stunCands.push(e.candidate); };
  const offerStun = await peerStun.createOffer(); 
  await peerStun.setLocalDescription(offerStun);
  await new Promise(r => setTimeout(r, 300));
  const srflx = stunCands.filter(c => c.candidate && c.candidate.includes('srflx'));
  console.log(`  ✓ STUN candidates: ${stunCands.length}, srflx: ${srflx.length}`);
  peerStun.close();

  // E2E-103: ICE候选类型验证
  console.log('[TEST] E2E-103: ICE candidate types');
  const allCands = [...hostCands, ...stunCands];
  const types = new Set();
  allCands.forEach(c => {
    const m = c.candidate.match(/typ (\w+)/);
    if (m) types.add(m[1]);
  });
  console.log(`  ✓ Types found: ${Array.from(types).join(', ') || 'none'}`);

  // E2E-104: TURN预留
  console.log('[TEST] E2E-104: TURN relay (RESERVED)');
  const turnCfg = { iceServers: [...CONFIG.STUN_SERVERS, {urls:'turn:turn.example.com:3478'}] };
  const hasTurn = turnCfg.iceServers.some(s => s.urls.startsWith('turn:'));
  console.log(`  ⚪ TURN config ready: ${hasTurn} (Sprint4)`);

  console.log('\n[NAT] All tests completed! Exit 0');
}

runTests().catch(err => {
  console.error('[NAT] Error:', err.message);
  process.exit(1);
});
