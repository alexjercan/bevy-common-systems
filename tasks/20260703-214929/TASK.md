# PROPOSAL (needs user): real audio assets + optional background music/ambience for example games

- STATUS: OPEN
- PRIORITY: 30
- TAGS: suggestion


PROPOSAL for the user -- NOT to be implemented autonomously; it needs assets
and/or a scope decision.

Every sound under `assets/sounds/` is a generated sine-blip placeholder
(`scripts/gen-placeholder-sounds.py`). They prove the wiring but are not nice to
listen to. Two related asks that both need you:

1. Real SFX assets. Drop real one-shot wav/ogg files at the same paths and no
   code changes are needed (see `assets/sounds/README.md`). Needs sourcing
   (freesound/CC0, a pack, or commissioned) and a licensing decision -- I should
   not commit third-party audio without you choosing the source/license.

2. Background music / ambience loop for the menu and/or during a run. This is
   out of scope for the current `audio` module on purpose: its doc says
   `SfxPlugin` is "SFX only, not music or a mixer". Delivering this means either
   a looping `AudioPlayer` wired directly in the example, or a small new
   `MusicPlugin` concern in the crate (crossfade menu<->gameplay, respect a
   master volume). Plus an actual music asset + license.

Decision needed: whether you want real assets sourced (and from where/what
license), and whether background music is in scope -- and if so, example-local
loop vs a new crate module.
