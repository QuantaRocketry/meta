import {
  Field,
  FieldGroup,
  FieldLabel,
  FieldSeparator,
} from "@/components/ui/field";
import { Input } from "@/components/ui/input";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Switch } from "@/components/ui/switch";
import type { RadioConfig } from "@/generated/qcp/RadioConfig";
import {
  BANDWIDTHS,
  CODING_RATES,
  CRC_KINDS,
  DATA_CODINGS,
  MODULATIONS,
  PREAMBLE_PATTERNS,
  PULSE_SHAPES,
  SPREADING_FACTORS,
  formToRadio,
  radioFormSchema,
  radioToForm,
  type RadioFormValues,
} from "@/lib/radio-form";
import { zodResolver } from "@hookform/resolvers/zod";
import { useEffect } from "react";
import {
  Controller,
  FormProvider,
  useForm,
  useFormContext,
  useWatch,
  type Path,
} from "react-hook-form";

type Name = Path<RadioFormValues>;

function fieldError(errors: object, name: string): string | undefined {
  let node: unknown = errors;
  for (const part of name.split(".")) {
    node = (node as Record<string, unknown> | undefined)?.[part];
  }
  return (node as { message?: string } | undefined)?.message;
}

function ErrorText({ name }: { name: Name }) {
  const { formState } = useFormContext<RadioFormValues>();
  const message = fieldError(formState.errors, name);
  return message ? (
    <p className="text-sm text-destructive">{message}</p>
  ) : null;
}

function NumberField({
  name,
  label,
  step,
}: {
  name: Name;
  label: string;
  step?: number;
}) {
  const { register } = useFormContext<RadioFormValues>();
  return (
    <Field>
      <FieldLabel htmlFor={name}>{label}</FieldLabel>
      <Input
        id={name}
        type="number"
        step={step}
        {...register(name, { valueAsNumber: true })}
      />
      <ErrorText name={name} />
    </Field>
  );
}

function TextField({ name, label }: { name: Name; label: string }) {
  const { register } = useFormContext<RadioFormValues>();
  return (
    <Field>
      <FieldLabel htmlFor={name}>{label}</FieldLabel>
      <Input id={name} {...register(name)} />
      <ErrorText name={name} />
    </Field>
  );
}

function SelectField({
  name,
  label,
  options,
}: {
  name: Name;
  label: string;
  options: Readonly<Record<string, string>>;
}) {
  const { control } = useFormContext<RadioFormValues>();
  return (
    <Controller
      name={name}
      control={control}
      render={({ field }) => (
        <Field>
          <FieldLabel>{label}</FieldLabel>
          <Select
            name={field.name}
            value={String(field.value)}
            onValueChange={field.onChange}
          >
            <SelectTrigger>
              <SelectValue />
            </SelectTrigger>
            <SelectContent position={"popper"}>
              <SelectGroup>
                {Object.entries(options).map(([value, text]) => (
                  <SelectItem key={value} value={value}>
                    {text}
                  </SelectItem>
                ))}
              </SelectGroup>
            </SelectContent>
          </Select>
        </Field>
      )}
    />
  );
}

function SwitchField({ name, label }: { name: Name; label: string }) {
  const { control } = useFormContext<RadioFormValues>();
  return (
    <Controller
      name={name}
      control={control}
      render={({ field }) => (
        <Field orientation="horizontal">
          <FieldLabel htmlFor={name}>{label}</FieldLabel>
          <Switch
            id={name}
            checked={Boolean(field.value)}
            onCheckedChange={field.onChange}
          />
        </Field>
      )}
    />
  );
}

function LoraFields() {
  const { control } = useFormContext<RadioFormValues>();
  const implicit = useWatch({ control, name: "lora.implicitHeader" });
  return (
    <>
      <SelectField
        name="lora.spreadingFactor"
        label="Spreading factor"
        options={SPREADING_FACTORS}
      />
      <SelectField
        name="lora.bandwidth"
        label="Bandwidth"
        options={BANDWIDTHS}
      />
      <SelectField
        name="lora.codingRate"
        label="Coding rate"
        options={CODING_RATES}
      />
      <NumberField name="lora.preambleLength" label="Preamble (symbols)" />
      <TextField name="lora.syncWord" label="Sync word (hex)" />
      <FieldSeparator />
      <SwitchField name="lora.implicitHeader" label="Implicit header" />
      {implicit && (
        <NumberField name="lora.payloadLength" label="Payload length (bytes)" />
      )}
      <SwitchField name="lora.crcEnabled" label="Payload CRC" />
      <SwitchField name="lora.invertIq" label="Invert IQ" />
    </>
  );
}

function GfskFields() {
  const { control } = useFormContext<RadioFormValues>();
  const variable = useWatch({ control, name: "gfsk.variableLength" });
  const crc = useWatch({ control, name: "gfsk.crc" });
  return (
    <>
      <NumberField name="gfsk.bitRate" label="Bit rate (bit/s)" />
      <NumberField name="gfsk.freqDeviation" label="Deviation (Hz)" />
      <SelectField
        name="gfsk.pulseShape"
        label="Pulse shape"
        options={PULSE_SHAPES}
      />
      <SelectField
        name="gfsk.preamblePattern"
        label="Preamble pattern"
        options={PREAMBLE_PATTERNS}
      />
      <NumberField name="gfsk.preambleLength" label="Preamble (bytes)" />
      <TextField name="gfsk.syncWord" label="Sync word (hex bytes)" />
      <FieldSeparator />
      <SwitchField name="gfsk.variableLength" label="Variable length" />
      {!variable && (
        <NumberField name="gfsk.fixedLength" label="Payload length (bytes)" />
      )}
      <SelectField name="gfsk.crc" label="CRC" options={CRC_KINDS} />
      {crc === "Crc16" && (
        <>
          <TextField name="gfsk.polynomial" label="CRC polynomial (hex)" />
          <TextField name="gfsk.initial" label="CRC initial value (hex)" />
          <SwitchField name="gfsk.invertOutput" label="Invert CRC output" />
          <SwitchField name="gfsk.includeLength" label="CRC covers length byte" />
        </>
      )}
      <SelectField name="gfsk.coding" label="Data coding" options={DATA_CODINGS} />
    </>
  );
}

/**
 * Edits a `RadioConfig`. Valid edits are reported through `onChange` as they
 * happen, and `value` only seeds the form, so remount it (with a `key`) to
 * load a different config.
 */
export function RadioParametersForm({
  value,
  onChange,
}: {
  value: RadioConfig;
  onChange: (value: RadioConfig) => void;
}) {
  const form = useForm<RadioFormValues>({
    resolver: zodResolver(radioFormSchema),
    defaultValues: radioToForm(value),
  });
  const modulation = useWatch({ control: form.control, name: "modulation" });

  useEffect(() => {
    const subscription = form.watch((values) => {
      const parsed = radioFormSchema.safeParse(values);
      if (parsed.success) {
        onChange(formToRadio(parsed.data));
      }
    });
    return () => subscription.unsubscribe();
  }, [form, onChange]);

  return (
    <FormProvider {...form}>
      <form onSubmit={(e) => e.preventDefault()}>
        <FieldGroup className="w-full max-w-xs">
          <NumberField name="frequencyMhz" label="Frequency (MHz)" step={0.001} />
          <SelectField
            name="modulation"
            label="Modulation"
            options={MODULATIONS}
          />
          <FieldSeparator />
          {modulation === "Lora" ? <LoraFields /> : <GfskFields />}
        </FieldGroup>
      </form>
    </FormProvider>
  );
}
