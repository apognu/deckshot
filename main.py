import asyncio
import os
import pathlib
import shutil
import subprocess

PLUGIN_DIR = pathlib.Path(__file__).parent.resolve()
CONFIG_DIR = pathlib.Path("/home/deck/.config/deckshot")
CONFIG_FILE = CONFIG_DIR / "deckshot.yml"

PROCESS = None


class Plugin:
    async def _main(self):
        os.makedirs(CONFIG_DIR, exist_ok=True)

        if not os.path.exists(CONFIG_FILE):
            shutil.copyfile(PLUGIN_DIR / "deckshot.yml", CONFIG_FILE)
            os.chmod(CONFIG_FILE, 0o0600)

        await self.start(self)

        while True:
            await asyncio.sleep(1)

    async def _unload(self):
        await self.stop(self)

    async def start(self):
        global PROCESS

        if not await self.is_running(self):
            PROCESS = subprocess.Popen(
                [
                    str(PLUGIN_DIR / "bin" / "deckshot"),
                    "-c",
                    CONFIG_FILE,
                ]
            )

    async def stop(self):
        global PROCESS

        if await self.is_running(self):
            PROCESS.terminate()

        PROCESS = None

    async def toggle(self):
        if await self.is_running(self):
            await self.stop(self)
        else:
            await self.start(self)

    async def is_running(self):
        global PROCESS

        if PROCESS is None:
            return False

        return PROCESS.poll() is None
