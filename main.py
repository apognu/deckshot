import asyncio
import os
import pathlib
import logging
import shutil
import subprocess
import sys

PLUGIN_DIR = pathlib.Path(__file__).parent.resolve()
CONFIG_DIR = pathlib.Path("/home/deck/.config/deckshot")
CONFIG_FILE = CONFIG_DIR / "deckshot.yml"

sys.path.insert(0, str(PLUGIN_DIR / "py_modules"))

import yaml


def log(message):
    logging.info(f"[deckshot] {message}")


class Plugin:
    process = None

    async def _main(self):
        os.makedirs(CONFIG_DIR, exist_ok=True)

        if not os.path.exists(CONFIG_FILE):
            shutil.copyfile(PLUGIN_DIR / "deckshot.yml", CONFIG_FILE)
            os.chmod(CONFIG_FILE, 0o0600)

        config = await self.get_config(self)

        if config and config.get("enabled", True):
            await self.start(self)

        while True:
            await asyncio.sleep(1)

    async def _unload(self):
        await self.stop(self)

    async def start(self):
        if not await self.is_running(self):
            log("starting backend service")

            self.process = subprocess.Popen(
                [
                    str(PLUGIN_DIR / "bin" / "deckshot"),
                    "-c",
                    CONFIG_FILE,
                ]
            )

    async def stop(self):
        if await self.is_running(self):
            log("stopping backend service")

            self.process.terminate()

        self.process = None

    async def toggle(self):
        config = await self.get_config(self)
        config["enabled"] = not await self.is_running(self)

        await self.write_config(self, config)

        if await self.is_running(self):
            await self.stop(self)
        else:
            await self.start(self)

    async def is_running(self):
        if self.process is None:
            return False

        return self.process.poll() is None

    async def get_config(self):
        with open(CONFIG_FILE) as f:
            return yaml.safe_load(f)

    async def write_config(self, config):
        with open(CONFIG_FILE, "w") as fw:
            yaml.safe_dump(config, fw)
