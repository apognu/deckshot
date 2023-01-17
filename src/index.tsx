import {
  Router,
  definePlugin,
  DialogButton,
  Field,
  PanelSection,
  PanelSectionRow,
  ServerAPI,
  staticClasses,
  ToggleField
} from "decky-frontend-lib";
import { VFC, useState, useEffect } from "react";
import { FaCamera } from "react-icons/fa";

type Config = {
  uploader: { [name: string]: any }
};

const Status: VFC<{ api: ServerAPI }> = ({ api }) => {
  const [status, setStatus] = useState(false);

  const fetchStatus = async () => {
    const result = await api?.callPluginMethod<any, boolean>("is_running", {});

    if (result?.success) {
      setStatus(result.result);
    }
  };

  useEffect(() => {
    fetchStatus();

    const interval = setInterval(() => {
      fetchStatus();
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  const doToggle = async (oldStatus: boolean) => {
    setStatus(!oldStatus);

    await api?.callPluginMethod("toggle", {});
    await fetchStatus();
  };

  return (
    <PanelSectionRow>
      <ToggleField
        label="Deckshot status"
        description={`Deckshot is ${status ? "running" : "stopped"}`}
        checked={status}
        onChange={() => doToggle(status)}
      />
    </PanelSectionRow>
  );
};

const Content: VFC<{ api: ServerAPI }> = ({ api }) => {
  const [config, setConfig] = useState<Config | null>(null);

  const getConfig = async () => {
    const result = await api?.callPluginMethod<any, Config | null>("get_config", {});

    if (result?.success) {
      setConfig(result.result)
    }
  }

  useEffect(() => {
    getConfig();
  }, []);

  return (
    <PanelSection title="Configuration">
      <Status api={api} />

      <PanelSectionRow>
        <div style={{ padding: "8px 0", fontSize: "0.8em" }}>
          Deckshot has to be configured manually for now, refer to the instructions from the repository for details.
        </div>

        <DialogButton onClick={() => Router.NavigateToExternalWeb("https://github.com/apognu/deckshot#readme")}>
          Open instructions
        </DialogButton>
      </PanelSectionRow>

      <PanelSectionRow>
        <Field
          label="Uploader"
          description={config?.uploader?.kind ?? "N/A"}
        />
      </PanelSectionRow>
    </PanelSection>
  );
};

export default definePlugin((serverApi: ServerAPI) => {
  return {
    title: <div className={staticClasses.Title}>Deckshot</div>,
    content: <Content api={serverApi} />,
    icon: <FaCamera />,
  };
});
