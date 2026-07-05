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
import random
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
    # 11_overload events (menu_select, game_over and level_up are shared).
    "vent": (330.0, 0.12, 0.18),
    "alarm": (1200.0, 0.18, 0.22),
}

# 14_breach combat cues, given more punch than a pure sine: noise bursts for the
# gun / kill and pitch sweeps for the rest, all with a fast decay envelope so they
# read as impacts rather than musical blips. `menu_select` / `game_over` stay shared
# with the other games (generic UI cues).
#   name -> (kind, freq_start, freq_end, duration, amp)
# kind is "noise" (freqs ignored) or "sweep" (a sine gliding freq_start -> freq_end).
BREACH_SOUNDS = {
    "breach_shoot": ("noise", 0.0, 0.0, 0.09, 0.30),
    "breach_hit": ("sweep", 620.0, 320.0, 0.06, 0.24),
    "breach_kill": ("noise", 0.0, 0.0, 0.24, 0.30),
    "breach_hurt": ("sweep", 150.0, 90.0, 0.22, 0.28),
    "breach_wave": ("sweep", 300.0, 780.0, 0.34, 0.22),
    "breach_pickup": ("sweep", 680.0, 1300.0, 0.13, 0.22),
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


def render_fx(kind, f0, f1, duration, amp):
    """A punchier placeholder: a noise burst or a pitch-sweeping sine, with a fast
    decay so it reads as an impact. Deterministic (the noise RNG is seeded by name
    length via the caller, so regenerating gives identical bytes)."""
    total = int(SAMPLE_RATE * duration)
    fade = max(1, int(SAMPLE_RATE * 0.003))
    frames = bytearray()
    phase = 0.0
    for i in range(total):
        t = i / total if total else 0.0
        # Fast (quadratic) decay for the body, short linear fade-in to kill the click.
        env = amp * (1.0 - t) ** 2
        if i < fade:
            env *= i / fade
        if kind == "noise":
            sample = env * random.uniform(-1.0, 1.0)
        else:  # "sweep": a sine gliding from f0 to f1
            freq = f0 + (f1 - f0) * t
            phase += 2.0 * math.pi * freq / SAMPLE_RATE
            sample = env * math.sin(phase)
        sample = max(-1.0, min(1.0, sample))
        frames += struct.pack("<h", int(sample * 32767.0))
    return bytes(frames)


def write_wav(path, data):
    with wave.open(path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(SAMPLE_RATE)
        w.writeframes(data)
    print("wrote", path)


def main():
    out_dir = os.path.join(os.path.dirname(__file__), "..", "assets", "sounds")
    out_dir = os.path.normpath(out_dir)
    os.makedirs(out_dir, exist_ok=True)

    for name, (freq, duration, amp) in SOUNDS.items():
        write_wav(os.path.join(out_dir, name + ".wav"), render(freq, duration, amp))

    for name, (kind, f0, f1, duration, amp) in BREACH_SOUNDS.items():
        # Seed the noise per name so regenerating is byte-identical.
        random.seed(name)
        write_wav(os.path.join(out_dir, name + ".wav"), render_fx(kind, f0, f1, duration, amp))


if __name__ == "__main__":
    main()
