import {
  definePlugin,
  PanelSection,
  PanelSectionRow,
  ServerAPI,
  staticClasses,
  ToggleField
} from "decky-frontend-lib";
import { VFC, useState, useEffect } from "react";
import { FaCamera } from "react-icons/fa";

var server: ServerAPI | undefined = undefined;

const Status: VFC = () => {
  const [status, setStatus] = useState(false);

  const fetchStatus = async () => {
    const result = await server?.callPluginMethod<any, boolean>("is_running", {});

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
  }, [])

  const doToggle = async (oldStatus: boolean) => {
    setStatus(!oldStatus);

    await server?.callPluginMethod("toggle", {});
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

const Content: VFC<{ serverAPI: ServerAPI }> = ({ }) => {
  return (
    <PanelSection title="Configuration">
      <Status />

      <PanelSectionRow>
        <div style={{ padding: "8px 0", fontSize: "0.8em" }}>
          Deckshot has to be configured manually for now, refer to the instructions from the repository for details.
        </div>
      </PanelSectionRow>
    </PanelSection>
  );
};

export default definePlugin((serverApi: ServerAPI) => {
  server = serverApi

  return {
    title: <div className={staticClasses.Title}>Deckshot</div>,
    content: <Content serverAPI={serverApi} />,
    icon: <FaCamera />
  };
});
