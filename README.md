# Omni-STT

**A cross-platform, modular AI-based application for real-time subtitling.**

Omni-STT bridges the gap between your voice and the screen, providing real-time AI-powered transcription directly over any active window. Designed with a modular architecture, it currently supports Soniox as the primary transcription provider, with plans for extensible support for other AI models.

Key Features
* Transparent Overlay: Subtitles are displayed on top of all windows without interfering with your work.
* Always on Top: The subtitle window remains visible regardless of active application.
* Mouse Passthrough: Interact with windows beneath the subtitles via click-through support.
* High Performance: Written in Rust with a focus on low-latency resource management (RAII).
* Highly Customizable: Adjust font size, text color, and layout to suit your workflow.
* Modular Architecture: Easily switch between different AI providers as your needs change.

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
* Soniox: Integrated and ready to use.
* More providers (Vosk, Google, etc.) are coming soon!

### Support
If you encounter any issues or have feature requests, please check the [Issues section](https://github.com/eoftgge/omni_stt/issues) on GitHub.
