import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Badge } from "@/components/ui/badge";
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

export function LogsPage() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [error, setError] = useState<string | null>(null);

  async function fetchLogs() {
    try {
      const entries = await invoke<LogEntry[]>("get_recent_logs", {
        limit: 100,
      });
      setLogs(entries);
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }

  useEffect(() => {
    fetchLogs();
    const interval = setInterval(fetchLogs, 5000);
    return () => clearInterval(interval);
  }, []);

  return (
    <div className="flex flex-1 flex-col gap-4 p-6">
      <div className="flex items-center justify-between">
        <h1 className="text-lg font-semibold">Logs</h1>
        <span className="text-xs text-muted-foreground">
          Auto-refreshing every 5s
        </span>
      </div>

      {error && <p className="text-sm text-destructive">{error}</p>}

      {logs.length === 0 && !error && (
        <p className="text-sm text-muted-foreground">
          No logs yet. Logs appear when agents send requests to plugmux.
        </p>
      )}

      {logs.length > 0 && (
        <div className="rounded-md border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Time</TableHead>
                <TableHead>Env</TableHead>
                <TableHead>Method</TableHead>
                <TableHead>Agent</TableHead>
                <TableHead className="text-right">Duration</TableHead>
                <TableHead>Status</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {logs.map((log) => (
                <TableRow key={log.id}>
                  <TableCell className="whitespace-nowrap text-muted-foreground">
                    {formatTime(log.timestamp)}
                  </TableCell>
                  <TableCell>
                    <Badge variant="outline" className="font-mono text-xs">
                      {log.env_id}
                    </Badge>
                  </TableCell>
                  <TableCell className="font-mono">{log.method}</TableCell>
                  <TableCell className="text-muted-foreground">
                    {log.agent_info?.agent_id || "—"}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {log.duration_ms}ms
                  </TableCell>
                  <TableCell>
                    {log.error ? (
                      <Badge variant="destructive" className="text-xs">
                        error
                      </Badge>
                    ) : (
                      <Badge
                        variant="secondary"
                        className="bg-green-100 text-green-800 dark:bg-green-900 dark:text-green-200 text-xs"
                      >
                        ok
                      </Badge>
                    )}
                  </TableCell>
                </TableRow>
              ))}
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
