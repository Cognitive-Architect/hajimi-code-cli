/**
 * Child Peer Helper - 接收文件并计算SHA256
 * 与父进程通过process.on('message')通信
 */
const crypto = require('crypto');
const wrtc = require('@koush/wrtc');

const receivedChunks = [];
let expectedTotal = 0;
let fileHash = '';
let startTime = 0;

console.log('[CHILD] Worker started, wrtc loaded');

// Signal ready to parent
process.send({ type: 'ready' });

process.on('message', async (msg) => {
  try {
    switch (msg.type) {
      case 'offer':
        await handleOffer(msg.sdp);
        break;
      case 'ice-candidate':
        if (global.pc) {
          await global.pc.addIceCandidate(new wrtc.RTCIceCandidate(msg.candidate));
        }
        break;
    }
  } catch (err) {
    process.send({ type: 'error', error: err.message });
  }
});

async function handleOffer(offerSdp) {
  console.log('[CHILD] Received offer, creating peer connection...');

  const pc = new wrtc.RTCPeerConnection({
    iceServers: [{ urls: 'stun:stun.l.google.com:19302' }]
  });
  global.pc = pc;

  // ICE state monitoring
  pc.oniceconnectionstatechange = () => {
    console.log(`[CHILD] ICE state: ${pc.iceConnectionState}`);
  };

  // Send ICE candidates to parent
  pc.onicecandidate = (event) => {
    if (event.candidate) {
      process.send({ type: 'ice-candidate', candidate: event.candidate });
    }
  };

  // Handle incoming DataChannel
  pc.ondatachannel = (event) => {
    const channel = event.channel;
    console.log('[CHILD] DataChannel received');

    channel.onopen = () => {
      console.log('[CHILD] DataChannel open');
      startTime = Date.now();
    };

    channel.onmessage = (e) => {
      const data = JSON.parse(e.data);
      handleDataMessage(data, channel);
    };

    channel.onclose = () => {
      console.log('[CHILD] DataChannel closed');
    };
  };

  // Set remote description and create answer
  await pc.setRemoteDescription(new wrtc.RTCSessionDescription(offerSdp));
  const answer = await pc.createAnswer();
  await pc.setLocalDescription(answer);

  console.log('[CHILD] Sending answer to parent');
  process.send({ type: 'answer', sdp: pc.localDescription });
}

function handleDataMessage(msg, channel) {
  switch (msg.type) {
    case 'file-chunk':
      receivedChunks[msg.index] = Buffer.from(msg.data, 'base64');
      expectedTotal = msg.total;
      console.log(`[CHILD] Received chunk ${msg.index + 1}/${msg.total}`);

      // Send ACK
      channel.send(JSON.stringify({ type: 'ack', index: msg.index }));
      break;

    case 'file-complete':
      console.log('[CHILD] File complete marker received');
      fileHash = msg.sha256;
      assembleAndVerify();
      break;
  }
}

function assembleAndVerify() {
  // Assemble file
  const fileBuffer = Buffer.concat(receivedChunks.filter(Boolean));
  const duration = Date.now() - startTime;

  console.log(`[CHILD] Assembled ${fileBuffer.length} bytes in ${duration}ms`);

  // Compute SHA256
  const computedHash = crypto.createHash('sha256').update(fileBuffer).digest('hex');
  const match = computedHash === fileHash;

  console.log(`[CHILD] Expected SHA256: ${fileHash.slice(0, 16)}...`);
  console.log(`[CHILD] Computed SHA256: ${computedHash.slice(0, 16)}...`);
  console.log(`[CHILD] Hash match: ${match}`);

  // Report result to parent
  process.send({
    type: 'file-received',
    sha256: computedHash,
    expectedSha256: fileHash,
    match: match,
    size: fileBuffer.length,
    duration: duration
  });
}

// Keep process alive
setInterval(() => {}, 1000);
