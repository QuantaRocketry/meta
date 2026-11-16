import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { showError } from "@/utils/error";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

const CHANNELS = [0, 1, 2, 3, 4, 5, 6, 7];

/** Channel 0 is 15 kHz below the centre; channel n is 235 kHz + (n - 1) * 250 kHz above it. */
function carrierMhz(centerMhz: number, channel: number) {
  const offsetHz = channel === 0 ? -15_000 : 235_000 + (channel - 1) * 250_000;
  return centerMhz + offsetHz / 1e6;
}

/** Sends the eggtimer radio settings to a connected qcp device (e.g. the feather). */
export function EggtimerRadioForm() {
  const [centerMhz, setCenterMhz] = useState(921);
  const [channel, setChannel] = useState(0);

  const [status, setStatus] = useState<{ ok: boolean; message: string } | null>(
    null,
  );

  useEffect(() => {
    const unlisten = listen<{ ok: boolean; message: string }>(
      "radio_config_result",
      (e) => setStatus(e.payload),
    );
    return () => {
      unlisten.then((f) => f());
    };
  }, []);

  async function send() {
    setStatus({ ok: true, message: "Waiting for the device..." });
    await invoke("send_eggtimer_config", {
      centerFrequencyHz: Math.round(centerMhz * 1e6),
      channel,
    }).catch((e) => {
      setStatus(null);
      showError(e);
    });
  }

  return (
    <Card>
      <CardHeader>Eggtimer receiver</CardHeader>
      <CardContent>
        <FieldGroup>
          <Field>
            <FieldLabel>Centre frequency (MHz)</FieldLabel>
            <Input
              type="number"
              step={0.001}
              value={centerMhz}
              onChange={(e) => setCenterMhz(Number(e.target.value))}
            />
          </Field>
          <Field>
            <FieldLabel>Channel</FieldLabel>
            <Select
              value={String(channel)}
              onValueChange={(v) => setChannel(Number(v))}
            >
              <SelectTrigger>
                <SelectValue />
              </SelectTrigger>
              <SelectContent position={"popper"}>
                <SelectGroup>
                  {CHANNELS.map((c) => (
                    <SelectItem key={c} value={String(c)}>
                      {c}
                    </SelectItem>
                  ))}
                </SelectGroup>
              </SelectContent>
            </Select>
          </Field>
          <div className="flex items-center justify-between">
            <span>Carrier {carrierMhz(centerMhz, channel).toFixed(3)} MHz</span>
            <Button onClick={send}>Send to device</Button>
          </div>
          {status && (
            <div className={status.ok ? undefined : "text-destructive"}>
              {status.message}
            </div>
          )}
        </FieldGroup>
      </CardContent>
    </Card>
  );
}
