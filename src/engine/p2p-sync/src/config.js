/**
 * WebRTC STUN and Signaling Configuration
 * Google Public STUN servers + ICE policy
 */

const CONFIG = {
  STUN_SERVERS: [
    { urls: 'stun:stun.l.google.com:19302' },
    { urls: 'stun:stun1.l.google.com:19302' }
  ],
  ICE_POLICY: {
    iceTransportPolicy: 'all',
    iceCandidatePoolSize: 10,
    bundlePolicy: 'balanced',
    rtcpMuxPolicy: 'require'
  },
  TIMEOUT: 5000,
  SIGNALING: {
    HOST: 'localhost',
    PORT: 8080,
    PATH: '/signaling'
  },
  RECONNECT: {
    MAX_RETRIES: 5,
    DELAY: 3000
  },
  HEARTBEAT: {
    INTERVAL: 30000,
    TIMEOUT: 60000
  }
};

module.exports = CONFIG;
