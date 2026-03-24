import { useState, useEffect, useCallback } from "react";
import { toast } from "sonner";
import {
  detectAgents,
  getAgentRegistry,
  connectAgent,
  disconnectAgent,
  dismissAgent,
  type DetectedAgent,
} from "@/lib/commands";
import { useEvents } from "./useEvents";

export function useAgents() {
  const [agents, setAgents] = useState<DetectedAgent[]>([]);
  const [loading, setLoading] = useState(true);

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

  // Reload when a new agent makes its first call
  useEvents<{ agent_id: string; is_new: boolean }>(
    "agent_activity",
    (payload) => {
      if (payload.is_new) reload();
    },
  );

  const connectedAgents = agents.filter(
    (a) => a.status === "green" || a.status === "yellow",
  );
  const hasConnected = connectedAgents.length > 0;

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
    reload,
    connect: async (id: string) => {
      const prev = agents.find((a) => a.id === id);
      optimisticUpdate(id, "green");
      try {
        await connectAgent(id);
        await reload();
      } catch (e) {
        if (prev) optimisticUpdate(id, prev.status);
        toast.error("Failed to connect agent", {
          description: e instanceof Error ? e.message : String(e),
        });
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
        toast.error("Failed to disconnect agent", {
          description: e instanceof Error ? e.message : String(e),
        });
      }
    },
    dismiss: async (id: string) => {
      await dismissAgent(id);
      await reload();
    },
  };
}
