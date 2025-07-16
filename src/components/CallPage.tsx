import { useEffect, useRef, useState, useCallback } from "react";
import { useLocation } from "react-router-dom";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

export default function CameraComponent() {
  const videoRef = useRef<HTMLVideoElement>(null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const logEndRef = useRef<HTMLDivElement>(null);

  const [error, setError] = useState<string | null>(null);
  const [facingMode, setFacingMode] = useState<"user" | "environment">("user");
  const [logs, setLogs] = useState<string[]>([]);

  const location = useLocation();
  const { chatId } = location.state || {};

  const addLog = useCallback((message: string) => {
    const timestamp = new Date().toLocaleTimeString();
    setLogs((prev) => [...prev, `[${timestamp}] ${message}`]);
  }, []);

  useEffect(() => {
    logEndRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  useEffect(() => {
    let unlisten: (() => void) | null = null;

    (async () => {
      unlisten = await listen("received-video", async (event: any) => {
        const compressedBytes = event.payload;
        const uint8Array = new Uint8Array(compressedBytes);
        const view = new DataView(uint8Array.buffer);

        const w = view.getUint32(0, true);
        const h = view.getUint32(4, true);

        const canvas = canvasRef.current;
        if (!canvas) return;

        if (canvas.width !== w || canvas.height !== h) {
          canvas.width = w;
          canvas.height = h;
        }

        const imageBytes = uint8Array.slice(8);
        const blob = new Blob([imageBytes], { type: "image/jpeg" });
        if (blob.size === 0) return;

        const bitmap = await createImageBitmap(blob);
        const ctx = canvas.getContext("2d");
        ctx?.clearRect(0, 0, w, h);
        ctx?.drawImage(bitmap, 0, 0, w, h);
      });
    })();

    return () => {
      if (unlisten) unlisten();
    };
  }, []);

  useEffect(() => {
    let stream: MediaStream | null = null;
    let isMounted = true;

    (async () => {
      try {
        addLog(`Requesting camera with facingMode: ${facingMode}`);
        const constraints = {
          video: { facingMode },
        };
        const s = await navigator.mediaDevices.getUserMedia(constraints);
        if (!isMounted) {
          s.getTracks().forEach((t) => t.stop());
          addLog("Component unmounted before stream start; stopped tracks");
          return;
        }
        stream = s;
        if (videoRef.current) {
          videoRef.current.srcObject = s;
          addLog("Video stream started");
        }
        setError(null);
      } catch (err: any) {
        const msg = `getUserMedia error: ${err.message ?? err}`;
        addLog(msg);
        setError(`Failed to get media stream: ${err.message ?? err}`);
      }
    })();

    return () => {
      isMounted = false;
      if (stream) {
        stream.getTracks().forEach((t) => t.stop());
        addLog("Stopped camera stream on cleanup");
      }
    };
  }, [facingMode, addLog]);

  const captureFrame = useCallback(async () => {
    if (!videoRef.current) return;
    const video = videoRef.current;
    const canvas = document.createElement("canvas");
    canvas.width = video.videoWidth;
    canvas.height = video.videoHeight;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;
    ctx.drawImage(video, 0, 0);

    try {
      const blob = await new Promise<Blob | null>((resolve) =>
        canvas.toBlob(resolve, "image/jpeg", 0.6)
      );
      if (!blob) {
        const msg = "Failed to capture blob";
        addLog(msg);
        setError(msg);
        return;
      }

      const arrayBuffer = await blob.arrayBuffer();
      const img = new Uint8Array(arrayBuffer);
      const header = new Uint32Array([canvas.width, canvas.height]);
      const payload = new Uint8Array(8 + img.length);
      payload.set(new Uint8Array(header.buffer), 0);
      payload.set(img, 8);

      await invoke("handle_frame_rgba", {
        frame: {
          data: Array.from(payload),
          width: canvas.width,
          height: canvas.height,
          format: "jpeg",
        },
        chatId,
      });
      addLog("Frame sent to backend");
    } catch (e: any) {
      const msg = `captureFrame error: ${e?.message ?? e}`;
      addLog(msg);
      setError(`Error capturing frame: ${e?.message ?? e}`);
    }
  }, [chatId, addLog]);

  // Capture frame every 20ms (~50 FPS)
  useEffect(() => {
    const interval = setInterval(() => {
      captureFrame();
    }, 100);
    return () => clearInterval(interval);
  }, [captureFrame]);

  return (
    <div className="relative w-screen h-screen bg-black overflow-hidden text-neutral-100">
      {/* Fullscreen received video canvas in background */}
      <canvas
        ref={canvasRef}
        className="fixed top-0 left-0 w-full h-full object-cover z-0"
        style={{ backgroundColor: "black" }}
      />

      {/* Small phone camera video top-right */}
      <video
        ref={videoRef}
        autoPlay
        playsInline
        muted
        className="fixed top-4 right-4 w-40 h-30 rounded border-2 border-neutral-700 shadow-lg z-10 bg-black object-cover"
      />

      {/* Controls at bottom center */}
      <div className="fixed bottom-6 left-1/2 transform -translate-x-1/2 flex space-x-4 z-20">
        <button
          onClick={() =>
            setFacingMode((prev) => (prev === "user" ? "environment" : "user"))
          }
          className="px-4 py-2 bg-neutral-700 rounded hover:bg-neutral-600"
        >
          Switch Camera
        </button>
      </div>

      {/* Error message */}
      {error && (
        <div className="fixed bottom-20 left-1/2 transform -translate-x-1/2 text-red-400 z-20">
          {error}
        </div>
      )}

      {/* Logs panel */}
      <div className="fixed bottom-32 left-1/2 transform -translate-x-1/2 w-full max-w-3xl h-32 bg-neutral-800 text-xs overflow-y-auto p-2 rounded z-20">
        {logs.map((log, idx) => (
          <div key={idx} className="whitespace-pre-wrap">
            {log}
          </div>
        ))}
        <div ref={logEndRef} />
      </div>
    </div>
  );
}
