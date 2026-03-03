/**
 * Heartbeat E2E Test - 长连接验证 (60秒)
 * Tests that WebSocket connection remains alive with heartbeat mechanism
 */

const WebSocket = require('ws');
const { SignalingServer } = require('../src/p2p/signaling-server');

const TEST_DURATION = 60000; // 60 seconds
const HEARTBEAT_INTERVAL = 30000; // 30 seconds

async function runHeartbeatTest() {
  console.log('=== Heartbeat E2E Test Starting ===');
  console.log(`Test duration: ${TEST_DURATION}ms (60 seconds)`);
  
  // Start signaling server
  const server = new SignalingServer(19090).start();
  await new Promise(r => setTimeout(r, 500));
  
  const ws = new WebSocket('ws://localhost:19090');
  let pingCount = 0;
  let pongCount = 0;
  let connected = false;
  let disconnected = false;
  
  ws.on('open', () => {
    console.log('[E2E] Client connected to server');
    connected = true;
    
    // Send initial register message with valid jsonrpc 2.0
    ws.send(JSON.stringify({
      jsonrpc: '2.0',
      type: 'register',
      peerId: 'test-peer-001'
    }));
  });
  
  ws.on('ping', (data) => {
    pingCount++;
    console.log(`[E2E] Received ping #${pingCount} from server`);
    ws.pong(data);
    pongCount++;
    console.log(`[E2E] Sent pong #${pongCount} to server`);
  });
  
  ws.on('message', (data) => {
    const msg = JSON.parse(data.toString());
    console.log(`[E2E] Received message: ${msg.type}`);
  });
  
  ws.on('close', () => {
    console.log('[E2E] Connection closed');
    disconnected = true;
  });
  
  ws.on('error', (err) => {
    console.error('[E2E] WebSocket error:', err.message);
  });
  
  // Wait for test duration
  await new Promise(r => setTimeout(r, TEST_DURATION + 2000));
  
  // Verify results
  console.log('\n=== Test Results ===');
  console.log(`Connection established: ${connected}`);
  console.log(`Ping received: ${pingCount}`);
  console.log(`Pong sent: ${pongCount}`);
  console.log(`Unexpected disconnect: ${disconnected}`);
  console.log(`Connection alive after ${TEST_DURATION}ms: ${ws.readyState === WebSocket.OPEN}`);
  
  // Expected: at least 1 ping (30s interval over 60s = ~2 pings)
  const passed = connected && 
                 pingCount >= 1 && 
                 pongCount >= 1 && 
                 !disconnected && 
                 ws.readyState === WebSocket.OPEN;
  
  console.log(`\n=== Test ${passed ? 'PASSED' : 'FAILED'} ===`);
  
  // Cleanup
  ws.close();
  server.stop();
  
  process.exit(passed ? 0 : 1);
}

// Version check test
async function runVersionCheckTest() {
  console.log('\n=== Version Check E2E Test Starting ===');
  
  const server = new SignalingServer(19091).start();
  await new Promise(r => setTimeout(r, 500));
  
  const ws = new WebSocket('ws://localhost:19091');
  let errorReceived = false;
  
  ws.on('open', () => {
    console.log('[E2E] Client connected');
    
    // Send message with invalid jsonrpc version
    ws.send(JSON.stringify({
      jsonrpc: '1.0', // Invalid version
      type: 'register',
      peerId: 'test-peer-002'
    }));
  });
  
  ws.on('message', (data) => {
    const msg = JSON.parse(data.toString());
    console.log(`[E2E] Received: ${JSON.stringify(msg)}`);
    if (msg.type === 'error' && msg.code === 'E_SIGNALING_INVALID_JSONRPC') {
      errorReceived = true;
      console.log('[E2E] Invalid jsonrpc version correctly rejected');
    }
  });
  
  await new Promise(r => setTimeout(r, 1000));
  
  const passed = errorReceived;
  console.log(`\n=== Version Check Test ${passed ? 'PASSED' : 'FAILED'} ===`);
  
  ws.close();
  server.stop();
  
  return passed;
}

// Run tests
async function main() {
  const versionTestPassed = await runVersionCheckTest();
  await new Promise(r => setTimeout(r, 500));
  await runHeartbeatTest();
}

main().catch(err => {
  console.error('Test error:', err);
  process.exit(1);
});
