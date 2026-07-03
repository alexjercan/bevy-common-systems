#!/usr/bin/env python3
"""Generate tiny placeholder WAV sound effects for the example games.

Covers the events of both `06_fruitninja` and `07_orbit`. The real game sounds
are meant to be sourced by the user (see assets/sounds/README.md). Until then
this writes a short, quiet sine "blip" for each event so the examples run and
are audibly wired end to end. Each event gets its own frequency so they are
distinguishable by ear while testing.

Run from the repo root:  python3 scripts/gen-placeholder-sounds.py

It only uses the Python standard library (the `wave` and `struct` modules), so
it does not depend on ffmpeg/sox or any third-party package. Overwrite the
generated files with real assets at the same paths and no code changes are
needed.
"""

import math
import os
import struct
import wave

SAMPLE_RATE = 44100

# filename -> (frequency Hz, duration seconds, peak amplitude 0..1)
SOUNDS = {
    "menu_select": (660.0, 0.10, 0.20),
    "slice": (880.0, 0.09, 0.22),
    "splat": (300.0, 0.12, 0.22),
    "combo": (990.0, 0.14, 0.22),
    "golden": (1320.0, 0.16, 0.20),
    "bomb": (140.0, 0.30, 0.28),
    "game_over": (220.0, 0.60, 0.24),
    "launch": (520.0, 0.10, 0.16),
    # 07_orbit events (menu_select and game_over are shared with 06_fruitninja).
    "pickup": (1046.0, 0.10, 0.20),
    "hurt": (196.0, 0.22, 0.26),
    "level_up": (784.0, 0.20, 0.20),
}


def render(freq, duration, amp):
    """A mono 16-bit PCM sine with short linear fades to avoid click artifacts."""
    total = int(SAMPLE_RATE * duration)
    fade = max(1, int(SAMPLE_RATE * 0.008))
    frames = bytearray()
    for i in range(total):
        env = amp
        if i < fade:
            env *= i / fade
        elif i > total - fade:
            env *= (total - i) / fade
        sample = env * math.sin(2.0 * math.pi * freq * (i / SAMPLE_RATE))
        frames += struct.pack("<h", int(sample * 32767.0))
    return bytes(frames)


def main():
    out_dir = os.path.join(os.path.dirname(__file__), "..", "assets", "sounds")
    out_dir = os.path.normpath(out_dir)
    os.makedirs(out_dir, exist_ok=True)

    for name, (freq, duration, amp) in SOUNDS.items():
        path = os.path.join(out_dir, name + ".wav")
        with wave.open(path, "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(SAMPLE_RATE)
            w.writeframes(render(freq, duration, amp))
        print("wrote", path)


if __name__ == "__main__":
    main()
