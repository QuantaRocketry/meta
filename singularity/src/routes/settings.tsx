import Page from "@/utils/page";
import { Field, FieldGroup, FieldLabel } from "@/components/ui/field";
import { Button } from "@/components/ui/button";
import { invoke } from "@tauri-apps/api/core";
import { showError } from "@/utils/error";
import { useEffect, useState } from "react";
import {
  InputGroup,
  InputGroupAddon,
  InputGroupInput,
} from "@/components/ui/input-group";
import { EyeIcon, EyeOffIcon } from "lucide-react";
import { AppSettings } from "@/context/settings/AppSettings";

export default function Settings() {
  const [keyInput, setKeyInput] = useState<string | undefined>(undefined);
  const [showKey, setShowKey] = useState(false); // New state
  const [pageLoaded, setPageLoaded] = useState(false);

  async function handleSumbit(e: React.FormEvent) {
    e.preventDefault(); // Prevent page reload
    invoke("set_cesium_ion_token", { token: keyInput }).catch((e) => {
      showError(e);
    });
  }

  async function getSettings() {
    invoke("get_app_settings")
      .then((response) => {
        let settings = response as AppSettings;
        if (settings.cesium_api_key) {
          setKeyInput(settings.cesium_api_key);
        }

        setPageLoaded(true);
      })
      .catch((e) => {
        showError(
          <>
            <p>Failed to retrieve settings.</p>
            <p>{e}</p>
          </>,
        );
      });
  }

  useEffect(() => {
    getSettings();
  }, []);

  return (
    <Page title="Settings" loaded={pageLoaded}>
      <form className="gap-4 max-w-2xl" onSubmit={handleSumbit}>
        <FieldGroup>
          <FieldGroup>
            <Field className="w-fit">
              <FieldLabel htmlFor="input-cesium-api-key">
                Cesium API Key
              </FieldLabel>
              <InputGroup>
                <InputGroupInput
                  id="input-cesium-api-key"
                  type={showKey ? "text" : "password"}
                  value={keyInput}
                  onChange={(e) => setKeyInput(e.target.value)}
                  placeholder="sk-..."
                />
                <InputGroupAddon align="inline-end">
                  <Button
                    type="button"
                    variant="ghost"
                    size="icon"
                    className="hover:bg-transparent"
                    onClick={() => setShowKey(!showKey)}
                  >
                    {showKey ? <EyeIcon /> : <EyeOffIcon />}
                  </Button>
                </InputGroupAddon>
              </InputGroup>
            </Field>
          </FieldGroup>
          <Field>
            <div>
              <Button type="submit">Save</Button>
            </div>
          </Field>
        </FieldGroup>
      </form>
    </Page>
  );
}
