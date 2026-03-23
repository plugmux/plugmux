import { useState, useEffect, useCallback } from "react";
import { getEngineStatus, startEngine, stopEngine } from "@/lib/commands";
import { useEvents } from "./useEvents";

export function useEngine() {
  const [status, setStatus] = useState<"running" | "stopped" | "conflict">(
    "stopped",
  );

  useEffect(() => {
    getEngineStatus().then((s) =>
      setStatus(s as "running" | "stopped" | "conflict"),
    );
  }, []);

  useEvents("engine_status_changed", (payload: { status: string }) => {
    setStatus(payload.status as "running" | "stopped" | "conflict");
  });

  const toggle = useCallback(async () => {
    if (status === "running") {
      await stopEngine();
    } else {
      await startEngine();
    }
  }, [status]);

  return { status, toggle };
}
