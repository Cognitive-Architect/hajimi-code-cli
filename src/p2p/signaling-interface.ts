/**
 * WebRTC Signaling Interface - JSON-RPC 2.0
 */

export enum SignalingState {
  idle = "idle",
  connecting = "connecting",
  connected = "connected",
  failed = "failed"
}

export type IceCandidateType = "host" | "srflx" | "relay";

export interface RTCIceCandidateInit {
  candidate: string;
  sdpMid: string | null;
  sdpMLineIndex: number | null;
  usernameFragment: string | null;
}

export interface SignalingMessage {
  jsonrpc: "2.0";
  id: string;
  method: "offer" | "answer" | "ice";
  params: { sdp?: string; candidate?: RTCIceCandidateInit; type?: IceCandidateType; version?: string };
}

export interface SignalingError { code: number; message: string }

export interface SignalingClient {
  state: SignalingState;
  connect(peerId: string): Promise<void>;
  sendOffer(sdp: string): Promise<void>;
  sendAnswer(sdp: string): Promise<void>;
  sendIceCandidate(candidate: RTCIceCandidateInit, type: IceCandidateType): void;
}

export const SIGNALING_TIMEOUT_MS = 5000;

export const ErrorCodes = {
  E_SIGNALING_TIMEOUT: -32001, E_SIGNALING_REJECTED: -32002, E_SIGNALING_INVALID_SDP: -32003,
  E_ICE_GATHERING_FAILED: -32101, E_ICE_CONNECTION_FAILED: -32102, E_ICE_NO_CANDIDATES: -32103
} as const;
