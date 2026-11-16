import { ButtonGroup } from "@/components/ui/button-group";
import { SerialSelector } from "./serial-control/selector";
import { SerialSettingsDialog } from "./serial-control/settings-dialog";

export function SerialControlWidget() {
  return (
    <ButtonGroup>
      <SerialSelector />
      <SerialSettingsDialog />
    </ButtonGroup>
  );
}
