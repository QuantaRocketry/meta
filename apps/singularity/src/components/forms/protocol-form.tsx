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
import type { Channel } from "@/generated/qcp/Channel";
import type { Protocol } from "@/generated/qcp/Protocol";
import { useState } from "react";

type ProtocolName = "Quanta" | "Eggtimer" | "Uts";

const PROTOCOLS: Record<ProtocolName, string> = {
  Quanta: "Quanta",
  Eggtimer: "Eggtimer",
  Uts: "UTS",
};

const CHANNELS: Record<Channel, string> = {
  Ch0: "0",
  Ch1: "1",
  Ch2: "2",
  Ch3: "3",
  Ch4: "4",
  Ch5: "5",
  Ch6: "6",
  Ch7: "7",
};

const DEFAULT_CENTER_HZ = 921_000_000;

/** Channel 0 sits 15 kHz below the centre; channel n is 235 kHz + (n - 1) * 250 kHz above it. */
function channelOffsetHz(channel: Channel) {
  const n = Number(CHANNELS[channel]);
  return n === 0 ? -15_000 : 235_000 + (n - 1) * 250_000;
}

function nameOf(protocol: Protocol): ProtocolName {
  return typeof protocol === "string" ? protocol : "Eggtimer";
}

function SelectBox<K extends string>({
  value,
  options,
  onChange,
}: {
  value: K;
  options: Record<K, string>;
  onChange: (value: K) => void;
}) {
  return (
    <Select value={value} onValueChange={(v) => onChange(v as K)}>
      <SelectTrigger>
        <SelectValue />
      </SelectTrigger>
      <SelectContent position={"popper"}>
        <SelectGroup>
          {Object.entries<string>(options).map(([key, text]) => (
            <SelectItem key={key} value={key}>
              {text}
            </SelectItem>
          ))}
        </SelectGroup>
      </SelectContent>
    </Select>
  );
}

/** Edits a `Protocol`. Edits are reported through `onChange` as they happen. */
export function ProtocolForm({
  value,
  onChange,
}: {
  value: Protocol;
  onChange: (value: Protocol) => void;
}) {
  const eggtimer = typeof value === "object" ? value.Eggtimer : undefined;
  const [centerText, setCenterText] = useState(
    String((eggtimer?.center_frequency_hz ?? DEFAULT_CENTER_HZ) / 1e6),
  );

  function selectProtocol(name: ProtocolName) {
    if (name === "Eggtimer") {
      onChange({
        Eggtimer: { center_frequency_hz: DEFAULT_CENTER_HZ, channel: "Ch0" },
      });
      setCenterText(String(DEFAULT_CENTER_HZ / 1e6));
    } else {
      onChange(name);
    }
  }

  function updateEggtimer(center_frequency_hz: number, channel: Channel) {
    onChange({ Eggtimer: { center_frequency_hz, channel } });
  }

  return (
    <FieldGroup className="w-full max-w-xs">
      <Field>
        <FieldLabel>Protocol</FieldLabel>
        <SelectBox
          value={nameOf(value)}
          options={PROTOCOLS}
          onChange={selectProtocol}
        />
      </Field>
      {eggtimer && (
        <>
          <Field>
            <FieldLabel htmlFor="eggtimer-center">
              Centre frequency (MHz)
            </FieldLabel>
            <Input
              id="eggtimer-center"
              type="number"
              step={0.001}
              value={centerText}
              onChange={(e) => {
                setCenterText(e.target.value);
                const mhz = Number(e.target.value);
                if (e.target.value !== "" && Number.isFinite(mhz)) {
                  updateEggtimer(Math.round(mhz * 1e6), eggtimer.channel);
                }
              }}
            />
          </Field>
          <Field>
            <FieldLabel>Channel</FieldLabel>
            <SelectBox
              value={eggtimer.channel}
              options={CHANNELS}
              onChange={(channel) =>
                updateEggtimer(eggtimer.center_frequency_hz, channel)
              }
            />
          </Field>
          <p className="text-sm text-muted-foreground">
            Carrier{" "}
            {(
              (eggtimer.center_frequency_hz + channelOffsetHz(eggtimer.channel)) /
              1e6
            ).toFixed(3)}{" "}
            MHz
          </p>
        </>
      )}
    </FieldGroup>
  );
}
