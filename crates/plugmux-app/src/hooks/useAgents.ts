import { useState, useEffect, useCallback } from "react";
import {
  detectAgents,
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
    const detected = await detectAgents();
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
