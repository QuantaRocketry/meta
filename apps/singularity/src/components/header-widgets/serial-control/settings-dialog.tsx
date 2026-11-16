import {
  SerialSettingsForm,
  SerialSettingsValues,
} from "@/components/forms/serial-settings-form";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogClose,
  DialogContent,
  DialogFooter,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { invoke } from "@tauri-apps/api/core";
import { SettingsIcon } from "lucide-react";
import { useEffect, useState } from "react";

export function SerialSettingsDialog() {
  const [open, setOpen] = useState(false);
  const [serialSettings, setSerialSettings] = useState<SerialSettingsValues>();

  function onSubmit(data: SerialSettingsValues) {
    console.log(data);
    setSerialSettings(data);
    invoke("set_serial_settings", {
      settings: {
        connection_type: data.connectionType,
        protocol: data.protocol,
        baud_rate: Number(data.baudRate),
      },
    })
      .then(() => {
        setOpen(false);
      })
      .catch((e) => console.error(e));
  }

  useEffect(() => {
    invoke("get_serial_settings")
      .then((s) => {
        let settings = s as {
          connection_type: SerialSettingsValues["connectionType"];
          protocol: SerialSettingsValues["protocol"];
          baud_rate: number;
        };
        setSerialSettings({
          connectionType: settings.connection_type,
          protocol: settings.protocol,
          baudRate: settings.baud_rate.toString(),
        });
      })
      .catch((e) => console.error(e));
  }, []);

  return (
    <Dialog open={open} onOpenChange={(e) => setOpen(e)}>
      <DialogTrigger asChild>
        <Button
          variant="outline"
          size="icon"
          aria-label="Settings"
          onClick={() => {
            setOpen(true);
          }}
        >
          <SettingsIcon />
        </Button>
      </DialogTrigger>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader>
          <DialogTitle>Connection Settings</DialogTitle>
        </DialogHeader>
        <SerialSettingsForm
          id="form-serial-settings"
          onSubmit={onSubmit}
          defaultValues={serialSettings}
        />
        <DialogFooter>
          <DialogClose>
            <Button variant="outline">Cancel</Button>
          </DialogClose>
          <Button type="submit" form="form-serial-settings">
            Save
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
