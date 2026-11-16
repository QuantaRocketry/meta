import { useSettingsContext } from "@/context/SettingsProvider";
import { listen } from "@tauri-apps/api/event";
import { useEffect } from "react";
import { Toaster, toast } from "sonner";

interface DeviceReply {
  ok: boolean;
  message: string;
}

/** Shows the device's answer to a radio config as a toast, whichever page is open. */
export function DeviceReplyToaster() {
  const { theme } = useSettingsContext();

  useEffect(() => {
    const unlisten = listen<DeviceReply>("radio_config_result", ({ payload }) =>
      payload.ok ? toast.success(payload.message) : toast.error(payload.message),
    );
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  return <Toaster theme={theme === "dark" ? "dark" : "light"} richColors />;
}
