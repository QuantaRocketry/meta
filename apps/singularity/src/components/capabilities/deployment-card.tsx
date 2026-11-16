import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { Field, FieldGroup, FieldLabel, FieldSeparator } from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import { Switch } from "@/components/ui/switch";
import type { DeploymentSettings } from "@/context/settings/Device";

export function DeploymentCard({
  value,
  onChange,
}: {
  value: DeploymentSettings;
  onChange: (value: DeploymentSettings) => void;
}) {
  const set = <K extends keyof DeploymentSettings>(
    key: K,
    next: DeploymentSettings[K],
  ) => onChange({ ...value, [key]: next });

  const numeric = (key: "apogee_delay" | "main_altitude") => ({
    type: "number" as const,
    value: value[key] === 0 ? "" : value[key],
    onChange: (e: React.ChangeEvent<HTMLInputElement>) =>
      set(key, Number(e.target.value)),
  });

  return (
    <Card>
      <CardHeader>Deployment Configuration</CardHeader>
      <CardContent>
        <FieldGroup className="w-full max-w-xs">
          <Field orientation="horizontal">
            <FieldLabel htmlFor="apogee-event-toggle">
              Deploy Drogue at Apogee
            </FieldLabel>
            <Switch
              id="apogee-event-toggle"
              checked={value.apogee}
              onCheckedChange={(v) => set("apogee", v)}
            />
          </Field>
          <Field>
            <FieldLabel htmlFor="delay-input">Apogee Delay (seconds)</FieldLabel>
            <Input
              id="delay-input"
              placeholder="Enter Apogee Deployment Delay..."
              {...numeric("apogee_delay")}
            />
          </Field>

          <FieldSeparator />

          <Field orientation="horizontal">
            <FieldLabel htmlFor="main-event-toggle">Deploy Main</FieldLabel>
            <Switch
              id="main-event-toggle"
              checked={value.main}
              onCheckedChange={(v) => set("main", v)}
            />
          </Field>
          <Field>
            <FieldLabel htmlFor="main-alt-input">
              Main Deployment Altitude (metres)
            </FieldLabel>
            <Input
              id="main-alt-input"
              placeholder="Enter Main Deployment Altitude..."
              {...numeric("main_altitude")}
            />
          </Field>
        </FieldGroup>
      </CardContent>
    </Card>
  );
}
