import { useState, useEffect } from "react";
import { ExternalLink, RefreshCw } from "lucide-react";
import { getVersion } from "@tauri-apps/api/app";
import { check } from "@tauri-apps/plugin-updater";
import { useEngine } from "@/hooks/useEngine";
import { useConfig } from "@/hooks/useConfig";
import { getPort, setPort } from "@/lib/commands";
import { enable, disable, isEnabled } from "@tauri-apps/plugin-autostart";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import { Switch } from "@/components/ui/switch";
import { PermissionsSection } from "@/components/settings/PermissionsSection";
import { CustomServersSection } from "@/components/settings/CustomServersSection";

export function SettingsPage() {
  const { status, toggle } = useEngine();
  const { config } = useConfig();

  // Version
  const [version, setVersion] = useState("");
  useEffect(() => {
    getVersion().then(setVersion);
  }, []);

  // Update check
  const [updateStatus, setUpdateStatus] = useState<
    "idle" | "checking" | "available" | "latest" | "error"
  >("idle");
  const [updateVersion, setUpdateVersion] = useState("");

  async function checkForUpdate() {
    setUpdateStatus("checking");
    try {
      const update = await check();
      if (update) {
        setUpdateVersion(update.version);
        setUpdateStatus("available");
      } else {
        setUpdateStatus("latest");
      }
    } catch {
      setUpdateStatus("error");
    }
  }

  // Port
  const [port, setPortValue] = useState<number>(0);
  useEffect(() => {
    getPort().then(setPortValue);
  }, []);
  const handlePortChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const val = Number(e.target.value);
    if (!Number.isNaN(val) && val > 0 && val <= 65535) {
      setPortValue(val);
      setPort(val);
    }
  };

  // Autostart
  const [autostart, setAutostart] = useState(false);
  useEffect(() => {
    isEnabled().then(setAutostart);
  }, []);
  const handleAutostartToggle = async (checked: boolean) => {
    if (checked) {
      await enable();
    } else {
      await disable();
    }
    setAutostart(checked);
  };

  // Dark mode
  const [dark, setDark] = useState(() =>
    document.documentElement.classList.contains("dark"),
  );
  const handleDarkToggle = (checked: boolean) => {
    setDark(checked);
    if (checked) {
      document.documentElement.classList.add("dark");
      document.body.classList.add("dark");
      localStorage.setItem("plugmux-theme", "dark");
    } else {
      document.documentElement.classList.remove("dark");
      document.body.classList.remove("dark");
      localStorage.setItem("plugmux-theme", "light");
    }
  };

  const badgeVariant =
    status === "running"
      ? "default"
      : status === "conflict"
        ? "destructive"
        : "secondary";

  return (
    <div className="p-6">
      <div className="mb-6">
        <h1 className="text-2xl font-bold">Settings</h1>
        <p className="text-sm text-muted-foreground">
          Configure gateway, startup, and appearance.
        </p>
      </div>

      {/* Quick links */}
      <section className="flex gap-3">
        <a
          href="https://www.plugmux.com/docs"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Button variant="outline" size="sm">
            <ExternalLink className="mr-1.5 h-3.5 w-3.5" />
            Documentation
          </Button>
        </a>
        <a
          href="https://github.com/plugmux/plugmux/issues"
          target="_blank"
          rel="noopener noreferrer"
        >
          <Button variant="outline" size="sm">
            <ExternalLink className="mr-1.5 h-3.5 w-3.5" />
            Support
          </Button>
        </a>
      </section>

      <Separator className="my-6" />

      {/* Gateway */}
      <section className="space-y-4">
        <h2 className="text-lg font-semibold">Gateway</h2>
        <div className="flex items-center gap-3">
          <Badge variant={badgeVariant}>{status}</Badge>
          <Button size="sm" variant="outline" onClick={toggle}>
            {status === "running" ? "Stop" : "Start"}
          </Button>
        </div>
        <div className="flex items-center gap-3">
          <Label htmlFor="port">Port</Label>
          <Input
            id="port"
            type="number"
            className="w-28"
            min={1}
            max={65535}
            value={port || ""}
            onChange={handlePortChange}
          />
        </div>
      </section>

      <Separator className="my-6" />

      {/* Permissions */}
      <PermissionsSection permissions={config?.permissions} />

      <Separator className="my-6" />

      {/* Custom Servers */}
      <CustomServersSection />

      <Separator className="my-6" />

      {/* Startup */}
      <section className="space-y-4">
        <h2 className="text-lg font-semibold">Startup</h2>
        <div className="flex items-center justify-between">
          <Label htmlFor="autostart">Launch on login</Label>
          <Switch
            id="autostart"
            checked={autostart}
            onCheckedChange={handleAutostartToggle}
          />
        </div>
      </section>

      <Separator className="my-6" />

      {/* Appearance */}
      <section className="space-y-4">
        <h2 className="text-lg font-semibold">Appearance</h2>
        <div className="flex items-center justify-between">
          <Label htmlFor="dark-mode">Dark mode</Label>
          <Switch
            id="dark-mode"
            checked={dark}
            onCheckedChange={handleDarkToggle}
          />
        </div>
      </section>

      <Separator className="my-6" />

      {/* About */}
      <section className="space-y-3">
        <h2 className="text-lg font-semibold">About</h2>
        <div className="flex items-center gap-3">
          <span className="text-sm text-muted-foreground">
            plugmux v{version || "..."}
          </span>
          <Button
            variant="ghost"
            size="sm"
            className="h-7 text-xs"
            onClick={checkForUpdate}
            disabled={updateStatus === "checking"}
          >
            <RefreshCw
              className={`mr-1 h-3 w-3 ${updateStatus === "checking" ? "animate-spin" : ""}`}
            />
            Check for updates
          </Button>
          {updateStatus === "available" && (
            <Badge variant="default" className="text-xs">
              v{updateVersion} available
            </Badge>
          )}
          {updateStatus === "latest" && (
            <span className="text-xs text-green-600 dark:text-green-400">
              Up to date
            </span>
          )}
          {updateStatus === "error" && (
            <span className="text-xs text-muted-foreground">
              Could not check for updates
            </span>
          )}
        </div>
        <p className="text-xs text-muted-foreground">MIT License</p>
      </section>
    </div>
  );
}
