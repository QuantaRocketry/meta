import { ProtocolForm } from "@/components/forms/protocol-form";
import { RadioParametersForm } from "@/components/forms/radio-parameters-form";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import type { RadioSettings } from "@/context/settings/Device";
import { showError } from "@/utils/error";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

type View = "parameters" | "protocol";

/**
 * Configures the radio of a qcp device with its two packets: the waveform
 * (Radio Parameters) and the protocol that rides on it (Protocol).
 */
export function RadioCard({
  value,
  onChange,
}: {
  value: RadioSettings;
  onChange: (value: RadioSettings) => void;
}) {
  const [view, setView] = useState<View>("parameters");

  async function send() {
    const sent =
      view === "parameters"
        ? invoke("send_radio_config", { config: value.config })
        : invoke("send_protocol", { protocol: value.protocol });
    await sent.catch((e) => showError(String(e)));
  }

  return (
    <Card>
      <CardHeader>Radio Configuration</CardHeader>
      <CardContent>
        <Tabs value={view} onValueChange={(v) => setView(v as View)}>
          <TabsList>
            <TabsTrigger value="parameters">Radio Parameters</TabsTrigger>
            <TabsTrigger value="protocol">Protocol</TabsTrigger>
          </TabsList>
          <TabsContent value="parameters">
            <RadioParametersForm
              value={value.config}
              onChange={(config) => onChange({ ...value, config })}
            />
          </TabsContent>
          <TabsContent value="protocol">
            <ProtocolForm
              value={value.protocol}
              onChange={(protocol) => onChange({ ...value, protocol })}
            />
          </TabsContent>
        </Tabs>
        <div className="mt-4 flex justify-end">
          <Button onClick={send}>
            Send {view === "parameters" ? "radio config" : "protocol"}
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}
