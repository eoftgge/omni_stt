# Omni-STT

**A cross-platform, modular AI-based application for real-time subtitling.**

Omni-STT bridges the gap between your voice and the screen, providing real-time
AI-powered transcription directly over any active window. It ships with two
recognition engines — Soniox in the cloud and Vosk fully offline — behind a
provider abstraction that makes adding more straightforward.

Key Features
* Transparent Overlay: Subtitles are displayed on top of all windows without interfering with your work.
* Always on Top: The subtitle window remains visible regardless of active application.
* Mouse Passthrough: Interact with windows beneath the subtitles via click-through support.
* High Performance: Written in Rust with a focus on low-latency resource management (RAII).
* Highly Customizable: Adjust font size, text color, and layout to suit your workflow.
* Modular Architecture: Easily switch between different AI providers as your needs change.
* Offline Mode: Vosk runs entirely on your machine, with no network and no API key.

### Launch

For build and start, you need [Rust Compiler](https://rust-lang.org/tools/install/)

```terminaloutput
>>> git clone https://github.com/eoftgge/omni_stt.git
>>> cd omni_stt
>>> cargo build --release
```
**Note:** After building, you will find the executable in target/release/. Move it to your preferred directory.
Antivirus software may flag the executable due to its ability to draw overlays — this is normal.

### Releases
You can also download the latest pre-compiled binaries from the [GitHub Releases page](https://github.com/eoftgge/omni_stt/releases).

## Supported Providers

**Soniox** — cloud recognition, works out of the box. Needs an API key, which is
kept in the OS keychain (Windows Credential Manager, macOS Keychain, Linux Secret
Service) rather than in the config file.

**Vosk** — fully offline recognition. The engine library and the language model
are not bundled: together they run to tens of megabytes, and most people only
need one of the two providers. Setup below.

### Setting up Vosk

Vosk needs two things. Settings will tell you whether it found each of them.

1. **The engine library.** Download the build for your platform from the
   [Vosk releases](https://github.com/alphacep/vosk-api/releases) and unpack it.
   Put `libvosk` next to the Omni-STT executable, or point at it in
   *Settings → Speech Engine → Library*. On Windows keep the `libgcc`,
   `libstdc++` and `libwinpthread` DLLs from the same archive alongside it —
   `libvosk.dll` will not load without them.

2. **A language model.** Pick one from the
   [Vosk model list](https://alphacephei.com/vosk/models) and unpack it. Point at
   the unpacked folder — the one containing `am/` and `conf/` — in
   *Settings → Speech Engine → Model*.

Models range from about 50 MB to well over a gigabyte. The small ones load in
seconds and are usually enough for live subtitles.

## Support
If you encounter any issues or have feature requests, please check the [Issues section](https://github.com/eoftgge/omni_stt/issues) on GitHub.
