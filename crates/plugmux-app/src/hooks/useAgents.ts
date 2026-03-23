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

  const reload = useCallback(async () => {
    setLoading(true);
    const [detected, registry] = await Promise.all([
      detectAgents(),
      getAgentRegistry(),
    ]);
    // Sort by registry order (agents.json defines canonical order)
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

  return {
    agents,
    loading,
    connectedAgents,
    hasConnected,
    reload,
    connect: async (id: string) => {
      await connectAgent(id);
      await reload();
    },
    disconnect: async (id: string, restore: boolean) => {
      await disconnectAgent(id, restore);
      await reload();
    },
    dismiss: async (id: string) => {
      await dismissAgent(id);
      await reload();
    },
  };
}
