import {
  Field,
  FieldGroup,
  FieldSeparator,
  FieldTitle,
} from "@/components/ui/field";
import {
  Select,
  SelectContent,
  SelectGroup,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { zodResolver } from "@hookform/resolvers/zod";
import { Controller, useForm, useWatch } from "react-hook-form";
import * as z from "zod";

const baud_rates = ["9600", "115200", "2000000"];

const connection_types = [
  { value: "usb", label: "USB" },
  { value: "serial", label: "Serial" },
] as const;

const protocols = [
  { value: "ascii", label: "ASCII" },
  { value: "qcp", label: "QCP" },
] as const;

const serialSettingsFormSchema = z.object({
  protocol: z.enum(["ascii", "qcp"]),
  connectionType: z.enum(["usb", "serial"]),
  baudRate: z.enum(baud_rates),
});

export type SerialSettingsValues = z.infer<typeof serialSettingsFormSchema>;

export function SerialSettingsForm({
  id,
  onSubmit,
  defaultValues,
}: {
  id: string;
  onSubmit: (data: SerialSettingsValues) => void;
  defaultValues?: Partial<SerialSettingsValues>;
}) {
  const form = useForm<SerialSettingsValues>({
    resolver: zodResolver(serialSettingsFormSchema),
    values: {
      connectionType: defaultValues?.connectionType || "usb",
      protocol: defaultValues?.protocol || "ascii",
      baudRate: defaultValues?.baudRate || "115200",
    },
  });

  const connectionType = useWatch({
    control: form.control,
    name: "connectionType",
  });

  return (
    <form id={id} onSubmit={form.handleSubmit(onSubmit)}>
      <FieldGroup>
        <Controller
          name="connectionType"
          control={form.control}
          render={({ field }) => (
            <Field orientation={"responsive"}>
              <FieldTitle>Connection</FieldTitle>
              <Select
                name={field.name}
                value={field.value}
                onValueChange={field.onChange}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select a connection type" />
                </SelectTrigger>
                <SelectContent position={"popper"}>
                  <SelectGroup>
                    {connection_types.map((ct) => (
                      <SelectItem key={ct.value} value={ct.value}>
                        {ct.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </Field>
          )}
        />
        {connectionType === "serial" && (
          <Controller
            name="baudRate"
            control={form.control}
            render={({ field }) => (
              <Field orientation={"responsive"}>
                <FieldTitle>Baud Rate</FieldTitle>
                <Select
                  name={field.name}
                  value={field.value}
                  onValueChange={field.onChange}
                >
                  <SelectTrigger>
                    <SelectValue placeholder="Select a baud rate" />
                  </SelectTrigger>
                  <SelectContent position={"popper"}>
                    <SelectGroup>
                      {baud_rates.map((br, index) => (
                        <SelectItem key={index} value={br.toString()}>
                          {br}
                        </SelectItem>
                      ))}
                    </SelectGroup>
                  </SelectContent>
                </Select>
              </Field>
            )}
          />
        )}
      </FieldGroup>
      <FieldSeparator className="my-4" />
      <FieldGroup>
        <Controller
          name="protocol"
          control={form.control}
          render={({ field }) => (
            <Field orientation={"responsive"}>
              <FieldTitle>Protocol</FieldTitle>
              <Select
                name={field.name}
                value={field.value}
                onValueChange={field.onChange}
              >
                <SelectTrigger>
                  <SelectValue placeholder="Select a protocol" />
                </SelectTrigger>
                <SelectContent position={"popper"}>
                  <SelectGroup>
                    {protocols.map((p) => (
                      <SelectItem key={p.value} value={p.value}>
                        {p.label}
                      </SelectItem>
                    ))}
                  </SelectGroup>
                </SelectContent>
              </Select>
            </Field>
          )}
        />
      </FieldGroup>
    </form>
  );
}
