import { useEffect, useState, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { Badge } from "@/components/ui/badge";
import { StatusDot, type StatusVariant } from "@/components/ui/status-dot";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";

interface AgentInfo {
  user_agent?: string;
  agent_id?: string;
  session_id: string;
}

interface LogEntry {
  id: string;
  timestamp: string;
  env_id: string;
  method: string;
  params_summary?: string;
  result_summary?: string;
  error?: string;
  duration_ms: number;
  agent_info?: AgentInfo;
}

function logStatus(log: LogEntry): { variant: StatusVariant; label: string } {
  if (log.error) return { variant: "error", label: log.error };
  if (log.duration_ms > 5000) return { variant: "warning", label: "Slow response" };
  return { variant: "success", label: "OK" };
}

export function LogsPage() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  const fetchLogs = useCallback(async () => {
    try {
      const entries = await invoke<LogEntry[]>("get_recent_logs", {
        limit: 100,
      });
      setLogs(entries);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }, []);

  useEffect(() => {
    fetchLogs();
    const unlisten = listen("log_added", () => {
      fetchLogs();
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, [fetchLogs]);

  return (
    <div className="flex flex-1 flex-col gap-4 overflow-hidden p-6">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold">Logs</h1>
        <span className="text-xs text-muted-foreground">
          Live
        </span>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {logs.length === 0 && !error && (
        <p className="text-sm text-muted-foreground">
          No logs yet. Logs appear when agents send requests to plugmux.
        </p>
      )}

      {logs.length > 0 && (
        <div className="min-h-0 flex-1 overflow-auto rounded-md border border-border/60">
          <Table>
            <TableHeader>
              <TableRow className="border-border/60 bg-muted/60 hover:bg-muted/60">
                <TableHead className="w-4 pl-2 pr-0"></TableHead>
                <TableHead className="w-[70px] pl-1.5 pr-1">Time</TableHead>
                <TableHead className="w-[80px] px-1">Env</TableHead>
                <TableHead className="px-2">Method</TableHead>
                <TableHead className="px-2">Agent</TableHead>
                <TableHead className="px-2 text-right">ms</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {logs.map((log) => {
                const { variant, label } = logStatus(log);
                return (
                  <TableRow key={log.id} className="border-border/40">
                    <TableCell className="w-4 pl-2 pr-0 text-center">
                      <StatusDot status={variant} label={label} />
                    </TableCell>
                    <TableCell className="w-[70px] whitespace-nowrap pl-1.5 pr-1 text-muted-foreground">
                      {formatTime(log.timestamp)}
                    </TableCell>
                    <TableCell className="w-[80px] px-1">
                      <Badge variant="outline" className="max-w-[72px] truncate font-mono text-xs">
                        {log.env_id}
                      </Badge>
                    </TableCell>
                    <TableCell className="max-w-[180px] truncate px-2 font-mono">
                      {log.method}
                    </TableCell>
                    <TableCell className="max-w-[100px] truncate px-2 text-muted-foreground">
                      {log.agent_info?.agent_id || "—"}
                    </TableCell>
                    <TableCell className="whitespace-nowrap px-2 text-right tabular-nums">
                      {log.duration_ms}
                    </TableCell>
                  </TableRow>
                );
              })}
            </TableBody>
          </Table>
        </div>
      )}
    </div>
  );
}

function formatTime(timestamp: string): string {
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString([], {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
    });
  } catch {
    return timestamp;
  }
}
