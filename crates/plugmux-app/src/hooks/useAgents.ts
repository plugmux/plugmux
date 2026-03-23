import { useState, useEffect, useCallback } from "react";
import {
  detectAgents,
  getAgentRegistry,
  connectAgent,
  disconnectAgent,
  dismissAgent,
  type DetectedAgent,
} from "@/lib/commands";

export function useAgents() {
  const [agents, setAgents] = useState<DetectedAgent[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  const reload = useCallback(async () => {
    setLoading(true);
    const [detected, registry] = await Promise.all([
      detectAgents(),
      getAgentRegistry(),
    ]);
    const orderMap = new Map(registry.map((a, i) => [a.id, i]));
    detected.sort((a, b) => {
      const ia = orderMap.get(a.id) ?? Infinity;
      const ib = orderMap.get(b.id) ?? Infinity;
      return ia - ib;
    });
    setAgents(detected);
    setLoading(false);
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  const connectedAgents = agents.filter(
    (a) => a.status === "green" || a.status === "yellow",
  );
  const hasConnected = connectedAgents.length > 0;

  // Optimistic update helper
  function optimisticUpdate(id: string, status: DetectedAgent["status"]) {
    setAgents((prev) =>
      prev.map((a) => (a.id === id ? { ...a, status } : a)),
    );
  }

  return {
    agents,
    loading,
    connectedAgents,
    hasConnected,
    error,
    clearError: () => setError(null),
    reload,
    connect: async (id: string) => {
      const prev = agents.find((a) => a.id === id);
      optimisticUpdate(id, "green");
      try {
        await connectAgent(id);
        await reload();
      } catch (e) {
        // Revert on error
        if (prev) optimisticUpdate(id, prev.status);
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    disconnect: async (id: string, restore: boolean) => {
      const prev = agents.find((a) => a.id === id);
      optimisticUpdate(id, "gray");
      try {
        await disconnectAgent(id, restore);
        await reload();
      } catch (e) {
        if (prev) optimisticUpdate(id, prev.status);
        setError(e instanceof Error ? e.message : String(e));
      }
    },
    dismiss: async (id: string) => {
      await dismissAgent(id);
      await reload();
    },
  };
}
