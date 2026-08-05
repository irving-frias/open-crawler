import Peer, { DataConnection } from 'peerjs';
import jsQR from 'jsqr';
import { open, readFile } from '@tauri-apps/plugin-fs';

export const P2P_CHUNK_SIZE = 64 * 1024;

export interface P2PProgress {
  transferred: number;
  total: number;
  percent: number;
}

export interface P2PSender {
  id: string;
  dispose(): void;
}

export interface ReceivedPackage {
  name: string;
  size: number;
}

function randomId(len: number): string {
  const chars = 'abcdefghijklmnopqrstuvwxyz0123456789';
  let out = '';
  for (let i = 0; i < len; i++) out += chars[Math.floor(Math.random() * chars.length)];
  return out;
}

function sleep(ms: number): Promise<void> {
  return new Promise((r) => setTimeout(r, ms));
}

/** A short but meaningful peer id for Open Crawler transfers. */
export function newPeerId(): string {
  return `ocp-${randomId(10)}`;
}

/** Extracts a PeerJS id from an `ocp2p:<id>` code, a LAN `/receive?peer=` URL
 * or a raw peer id. Returns `null` when nothing usable is found. */
export function parseP2PCode(input: string): string | null {
  const s = input.trim();
  if (!s) return null;
  if (s.startsWith('ocp2p:')) {
    const id = s.slice('ocp2p:'.length).trim();
    return isValidPeerId(id) ? id : null;
  }
  if (s.startsWith('http://') || s.startsWith('https://')) {
    try {
      const u = new URL(s);
      const p = u.searchParams.get('peer');
      if (p && isValidPeerId(p)) return p;
      const last = s.split('/').pop() ?? '';
      return isValidPeerId(last) ? last : null;
    } catch {
      return null;
    }
  }
  return isValidPeerId(s) ? s : null;
}

function isValidPeerId(id: string): boolean {
  return /^[A-Za-z0-9_-]{1,64}$/.test(id);
}

function newPeer(id: string, onError: (message: string) => void): Peer {
  const peer = new Peer(id);
  peer.on('error', (err) => onError(err.message || err.type));
  return peer;
}

/** Sends `data` over `conn`, pacing on PeerJS' buffer so a large file does not
 * overflow the WebRTC data channel. */
async function pacedSend(conn: DataConnection, data: unknown): Promise<void> {
  conn.send(data);
  // @ts-expect-error bufferSize exists on DataConnection (peerjs types miss it)
  const buffered = conn.bufferSize;
  if (typeof buffered === 'number' && buffered > 512 * 1024) {
    await sleep(10);
  }
}

/** Sender side: exposes a peer that streams an already-exported package file to
 * any receiver that connects and asks for it (request → header → ack → chunks → done). */
export function startP2PSender(
  filePath: string,
  fileName: string,
  fileSize: number,
  onStatus: (status: string) => void,
  onProgress: (p: P2PProgress) => void,
  onError: (message: string) => void
): P2PSender {
  const id = newPeerId();
  const peer = newPeer(id, onError);
  const conns = new Set<DataConnection>();

  peer.on('open', () => onStatus('ready'));

  peer.on('connection', (conn) => {
    conns.add(conn);
    conn.on('open', () => onStatus('connected'));
    conn.on('close', () => conns.delete(conn));
    conn.on('error', (e) => onError(e.message || e.type));
    conn.on('data', (data) => {
      void (async () => {
        try {
          const msg = data as { type?: string };
          if (msg && typeof msg === 'object' && msg.type === 'request') {
            onStatus('sending-header');
            conn.send({ type: 'header', name: fileName, size: fileSize });
          } else if (msg && typeof msg === 'object' && msg.type === 'ack') {
            onStatus('sending');
            const buf = await readFile(filePath);
            let sent = 0;
            for (let off = 0; off < buf.length; off += P2P_CHUNK_SIZE) {
              const chunk = buf.slice(off, Math.min(off + P2P_CHUNK_SIZE, buf.length));
              await pacedSend(conn, chunk);
              sent += chunk.byteLength;
              onProgress({ transferred: sent, total: buf.length, percent: (sent / buf.length) * 100 });
            }
            conn.send({ type: 'done' });
            onStatus('complete');
          }
        } catch (e) {
          onError(e instanceof Error ? e.message : String(e));
          conn.send({ type: 'error', message: e instanceof Error ? e.message : String(e) });
        }
      })();
    });
  });

  return {
    id,
    dispose: () => {
      for (const c of conns) c.close();
      peer.destroy();
    },
  };
}

/** Receiver side: connects to the sender's peer id and writes the incoming
 * package to `destPath`, then resolves with the package metadata. */
export function receiveP2P(
  peerId: string,
  destPath: string,
  onStatus: (status: string) => void,
  onProgress: (p: P2PProgress) => void,
  onError: (message: string) => void
): Promise<ReceivedPackage> {
  return new Promise((resolve, reject) => {
    const peer = newPeer(newPeerId(), onError);
    let settled = false;
    let file: Awaited<ReturnType<typeof open>> | null = null;

    const fail = (err: unknown) => {
      if (settled) return;
      settled = true;
      const msg = err instanceof Error ? err.message : String(err);
      onError(msg);
      void file?.close().catch(() => {});
      peer.destroy();
      reject(new Error(msg));
    };

    peer.on('open', () => {
      let conn: DataConnection | null = null;
      try {
        conn = peer.connect(peerId, { reliable: true });
      } catch (e) {
        return fail(e);
      }
      let header: { name: string; size: number } | null = null;
      let received = 0;
      let chain: Promise<void> = Promise.resolve();

      conn.on('open', () => onStatus('connected'));
      conn.on('error', (e) => fail(e.message || e.type));
      conn.on('close', () => {
        if (!settled) fail(new Error('connection closed before transfer finished'));
      });

      conn.on('data', (data) => {
        chain = chain.then(async () => {
          if (settled) return;
          if (data instanceof ArrayBuffer || ArrayBuffer.isView(data)) {
            if (!file) throw new Error('received data before header');
            const buf =
              data instanceof Uint8Array ? data : new Uint8Array(data as ArrayBuffer);
            await file.write(buf);
            received += buf.byteLength;
            onProgress({
              transferred: received,
              total: header?.size ?? 0,
              percent: header && header.size > 0 ? (received / header.size) * 100 : 0,
            });
            return;
          }
          const msg = data as { type?: string; name?: string; size?: number; message?: string };
          if (msg && typeof msg === 'object' && msg.type === 'header') {
            header = { name: String(msg.name ?? 'package.ocproj'), size: Number(msg.size ?? 0) };
            file = await open(destPath, { write: true, create: true, truncate: true });
            conn!.send({ type: 'ack' });
            onStatus('receiving');
            onProgress({ transferred: 0, total: header.size, percent: 0 });
          } else if (msg && typeof msg === 'object' && msg.type === 'done') {
            await file?.close();
            file = null;
            settled = true;
            onStatus('complete');
            peer.destroy();
            resolve({ name: header?.name ?? 'package.ocproj', size: received });
          } else if (msg && typeof msg === 'object' && msg.type === 'error') {
            throw new Error(String(msg.message ?? 'transfer error'));
          }
        }).catch(fail);
      });

      conn.send({ type: 'request' });
    });

    peer.on('error', fail);
  });
}

/** True when the current webview can do WebRTC (not available on WebKitGTK). */
export function webRtcAvailable(): boolean {
  return typeof window !== 'undefined' && typeof window.RTCPeerConnection === 'function';
}

/** Starts scanning frames from `video` for a QR code, resolving with the text
 * when one is found. Returns a stop function. */
export function startQrScanner(
  video: HTMLVideoElement,
  onResult: (text: string) => void,
  onError: (message: string) => void
): () => void {
  let raf = 0;
  let running = true;
  const canvas = document.createElement('canvas');

  const tick = () => {
    if (!running) return;
    raf = requestAnimationFrame(tick);
    if (video.readyState < 2 || video.videoWidth === 0) return;
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    ctx.drawImage(video, 0, 0, canvas.width, canvas.height);
    const imageData = ctx.getImageData(0, 0, canvas.width, canvas.height);
    const code = jsQR(imageData.data, imageData.width, imageData.height, {
      inversionAttempts: 'dontInvert',
    });
    if (code && code.data) {
      onResult(code.data);
      stop();
    }
  };

  const stop = () => {
    running = false;
    cancelAnimationFrame(raf);
    const stream = video.srcObject as MediaStream | null;
    if (stream) for (const track of stream.getTracks()) track.stop();
  };

  if (typeof navigator !== 'undefined' && navigator.mediaDevices?.getUserMedia) {
    navigator.mediaDevices
      .getUserMedia({ video: { facingMode: 'environment' } })
      .then((stream) => {
        if (!running) {
          stream.getTracks().forEach((t) => t.stop());
          return;
        }
        video.srcObject = stream;
        video.play().catch((e) => onError(String(e)));
        raf = requestAnimationFrame(tick);
      })
      .catch((e) => onError(String(e)));
  } else {
    onError('Camera not available');
  }

  return stop;
}
