/**
 * DataChannel Real Network E2E Test - FIND-032-01修复
 * 真实WebRTC双进程测试，无Mock，30秒熔断
 */
const { fork } = require('child_process');
const crypto = require('crypto');
const wrtc = require('@koush/wrtc');
const path = require('path');

const TIMEOUT_MS = 30000;
const TEST_FILE_SIZE = 128 * 1024; // 128KB test file
const STUN_SERVER = 'stun:stun.l.google.com:19302';

let passCount = 0, failCount = 0;
function assert(cond, msg) {
  if (cond) { passCount++; console.log(`  ✓ ${msg}`); }
  else { failCount++; console.error(`  ✗ ${msg}`); }
}

function timeoutPromise(ms, msg) {
  return new Promise((_, reject) => setTimeout(() => reject(new Error(msg)), ms));
}

async function runRealNetworkTest() {
  console.log('[E2E] Real Network WebRTC Test - FIND-032-01');
  console.log(`[E2E] STUN: ${STUN_SERVER}`);
  console.log('[E2E] Forking child process...');

  const childPath = path.join(__dirname, 'helpers', 'child-peer.js');
  const child = fork(childPath, [], { silent: false });

  let childReady = false;
  let iceConnected = false;
  let dataChannelOpen = false;
  let sha256Match = false;

  // Generate test file and expected hash
  const testFile = Buffer.alloc(TEST_FILE_SIZE, 0xAB);
  const expectedHash = crypto.createHash('sha256').update(testFile).digest('hex');
  console.log(`[E2E] Test file: ${TEST_FILE_SIZE} bytes, SHA256: ${expectedHash.slice(0, 16)}...`);

  // Create parent peer connection with STUN
  const pc = new wrtc.RTCPeerConnection({
    iceServers: [{ urls: STUN_SERVER }]
  });

  // Handle child process messages
  const childPromise = new Promise((resolve, reject) => {
    child.on('message', async (msg) => {
      switch (msg.type) {
        case 'ready':
          console.log('[E2E] Child process ready');
          childReady = true;
          break;

        case 'answer':
          console.log('[E2E] Received answer from child');
          await pc.setRemoteDescription(new wrtc.RTCSessionDescription(msg.sdp));
          break;

        case 'ice-candidate':
          try {
            await pc.addIceCandidate(new wrtc.RTCIceCandidate(msg.candidate));
          } catch (e) { /* ignore duplicate */ }
          break;

        case 'file-received':
          console.log(`[E2E] Child received file, computed SHA256: ${msg.sha256.slice(0, 16)}...`);
          sha256Match = msg.sha256 === expectedHash;
          assert(sha256Match, 'SHA256 match verified');
          resolve({ sha256Match, duration: msg.duration });
          break;

        case 'error':
          reject(new Error(`Child error: ${msg.error}`));
          break;
      }
    });

    child.on('error', reject);
    child.on('exit', (code) => {
      if (code !== 0 && !sha256Match) reject(new Error(`Child exited with code ${code}`));
    });
  });

  // ICE connection state monitoring
  pc.oniceconnectionstatechange = () => {
    console.log(`[E2E] ICE state: ${pc.iceConnectionState}`);
    if (pc.iceConnectionState === 'connected' || pc.iceConnectionState === 'completed') {
      iceConnected = true;
      console.log('[E2E] ICE connected!');
    }
  };

  // Handle ICE candidates
  pc.onicecandidate = (event) => {
    if (event.candidate) {
      child.send({ type: 'ice-candidate', candidate: event.candidate });
    }
  };

  // Create DataChannel
  const channel = pc.createDataChannel('fileTransfer', {
    ordered: true,
    maxRetransmits: 3
  });

  channel.onopen = () => {
    console.log('[E2E] DataChannel open!');
    dataChannelOpen = true;
    assert(channel.readyState === 'open', 'DataChannel readyState is open');

    // Send file in chunks
    const CHUNK_SIZE = 16384; // 16KB chunks for DataChannel
    const totalChunks = Math.ceil(testFile.length / CHUNK_SIZE);
    console.log(`[E2E] Sending ${totalChunks} chunks...`);

    for (let i = 0; i < totalChunks; i++) {
      const start = i * CHUNK_SIZE;
      const chunk = testFile.slice(start, start + CHUNK_SIZE);
      const msg = {
        type: 'file-chunk',
        index: i,
        total: totalChunks,
        data: chunk.toString('base64')
      };
      channel.send(JSON.stringify(msg));
    }

    // Send EOF marker
    channel.send(JSON.stringify({ type: 'file-complete', sha256: expectedHash }));
    console.log('[E2E] File transfer complete marker sent');
  };

  channel.onmessage = (event) => {
    const msg = JSON.parse(event.data);
    if (msg.type === 'ack') {
      console.log(`[E2E] Received ACK from child`);
    }
  };

  // Create and send offer
  const offer = await pc.createOffer();
  await pc.setLocalDescription(offer);
  console.log('[E2E] Created offer, sending to child...');

  // Wait for child to be ready before sending offer
  while (!childReady) await new Promise(r => setTimeout(r, 50));
  child.send({ type: 'offer', sdp: pc.localDescription });

  // Wait for completion with timeout
  try {
    const result = await Promise.race([
      childPromise,
      timeoutPromise(TIMEOUT_MS, `Test timeout after ${TIMEOUT_MS}ms`)
    ]);

    console.log(`[E2E] File transfer completed in ${result.duration}ms`);

    // Cleanup
    channel.close();
    pc.close();
    child.kill();

    assert(childReady, 'Child process was ready');
    assert(iceConnected, 'ICE connection established');
    assert(dataChannelOpen, 'DataChannel was opened');
    assert(result.sha256Match, 'SHA256 hash matches');

    console.log(`\n[E2E] Results: ${passCount} passed, ${failCount} failed`);
    if (failCount > 0) {
      console.error('[E2E] FAILED');
      process.exit(1);
    }
    console.log('[E2E] ALL PASSED');
    process.exit(0);

  } catch (err) {
    console.error(`[E2E] Test failed: ${err.message}`);
    channel.close?.();
    pc.close?.();
    child.kill();
    process.exit(1);
  }
}

runRealNetworkTest().catch(e => {
  console.error('[E2E] Fatal:', e);
  process.exit(1);
});
