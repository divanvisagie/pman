from __future__ import annotations

from dataclasses import dataclass
import os
import shutil
import subprocess


@dataclass(frozen=True)
class BridgeConfig:
    pman_bin: str = "pman"
    notes_dir: str | None = None


class PmanBridge:
    def __init__(self, config: BridgeConfig) -> None:
        self._config = config
        self._validate_binary(config.pman_bin)

    @staticmethod
    def _validate_binary(pman_bin: str) -> None:
        if os.path.sep in pman_bin:
            if os.path.exists(pman_bin):
                return
            raise FileNotFoundError(f"pman binary not found: {pman_bin}")
        if shutil.which(pman_bin) is None:
            raise FileNotFoundError(f"pman binary not found in PATH: {pman_bin}")

    def run(self, subcommand: str, *args: str) -> str:
        cmd: list[str] = [self._config.pman_bin, subcommand]
        if self._config.notes_dir:
            cmd.extend(["--notes-dir", self._config.notes_dir])
        cmd.extend(args)

        completed = subprocess.run(
            cmd,
            check=False,
            capture_output=True,
            text=True,
        )
        if completed.returncode != 0:
            stderr = completed.stderr.strip()
            stdout = completed.stdout.strip()
            details = stderr or stdout or "unknown error"
            raise RuntimeError(
                f"pman command failed ({completed.returncode}): {' '.join(cmd)}\n{details}"
            )
        return completed.stdout
