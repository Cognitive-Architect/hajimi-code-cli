/**
 * Laptop B Helper - 接收文件、修改、回传
 * 与父进程(two-laptops-sync.e2e.js)通信
 */
const crypto = require('crypto');
const wrtc = require('@koush/wrtc');

const receivedChunks = [];
let fileBuffer = null;
let pc = null;
let channel = null;

console.log('[CHILD-B] Worker started');
process.send({ type: 'ready' });

process.on('message', async (msg) => {
  try {
    if (msg.type === 'offer') await handleOffer(msg.sdp);
    if (msg.type === 'ice-candidate' && pc) await pc.addIceCandidate(new wrtc.RTCIceCandidate(msg.candidate));
  } catch (err) {
    process.send({ type: 'error', error: err.message });
  }
});

async function handleOffer(offerSdp) {
  pc = new wrtc.RTCPeerConnection({ iceServers: [{ urls: 'stun:stun.l.google.com:19302' }] });

  pc.onicecandidate = (e) => {
    if (e.candidate) process.send({ type: 'ice-candidate', candidate: e.candidate });
  };

  pc.ondatachannel = (e) => {
    channel = e.channel;
    console.log('[CHILD-B] DataChannel received');

    channel.onopen = () => console.log('[CHILD-B] DataChannel open');

    channel.onmessage = (evt) => {
      try {
        const data = JSON.parse(evt.data);
        handleDataMessage(data);
      } catch { /* raw data */ }
    };

    channel.onclose = () => console.log('[CHILD-B] DataChannel closed');
  };

  await pc.setRemoteDescription(new wrtc.RTCSessionDescription(offerSdp));
  const answer = await pc.createAnswer();
  await pc.setLocalDescription(answer);
  process.send({ type: 'answer', sdp: pc.localDescription });
}

function handleDataMessage(msg) {
  if (msg.type === 'file-chunk') {
    receivedChunks[msg.chunkIndex] = Buffer.from(msg.data, 'base64');
    channel.send(JSON.stringify({ type: 'chunk-ack', transferId: msg.transferId, chunkIndex: msg.chunkIndex }));
  }
  else if (msg.type === 'file-complete') {
    fileBuffer = Buffer.concat(receivedChunks.filter(Boolean));
    const computedHash = crypto.createHash('sha256').update(fileBuffer).digest('hex');
    console.log(`[CHILD-B] File received: ${fileBuffer.length} bytes`);
    process.send({ type: 'file-received', sha256: computedHash, size: fileBuffer.length, match: computedHash === msg.sha256 });
    modifyAndSendBack();
  }
}

function modifyAndSendBack() {
  const modified = Buffer.concat([fileBuffer, Buffer.from(' - Modified by B')]);
  const modifiedHash = crypto.createHash('sha256').update(modified).digest('hex');
  console.log(`[CHILD-B] Modified file: ${modified.length} bytes`);
  process.send({ type: 'modified-file', sha256: modifiedHash });
  if (channel?.readyState === 'open') {
    channel.send(JSON.stringify({ type: 'sync-complete', modifiedHash }));
  }
}

setInterval(() => {}, 1000);
