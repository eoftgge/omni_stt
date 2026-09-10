#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::{wgpu, WgpuConfiguration};
use eframe::egui::{IconData, ViewportBuilder};
use eframe::egui_wgpu::WgpuSetup;
use eframe::icon_data::from_png_bytes;
use eframe::wgpu::Backends;
use omni_stt::errors::OmniSttErrors;
use omni_stt::gui::app::SubtitlesApp;
use omni_stt::gui::fonts::setup_custom_fonts;
use omni_stt::settings::SettingsManager;
use omni_stt::setup_tracing;

use std::sync::Arc;

const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.png");

/// WARNING: A CRANK IS IN PLACE DUE TO INCORRECT DISPLAY OF THE TRANSPARENCY OVERLAY ON AMD RADEON INTEGRATED GRAPHICS CARDS.
fn select_adapter(adapters: &[wgpu::Adapter], _surface: Option<&wgpu::Surface<'_>>) -> Result<wgpu::Adapter, String> {
    if let Some(adapter) = adapters.iter().find(|a| {
        let info = a.get_info();
        let name = info.name.to_lowercase();
        // todo: DIRTY HACK!!! ADJUST IF POSSIBLE!!!
        (name.contains("amd") || name.contains("radeon")) && info.backend == wgpu::Backend::Gl
    }) {
        return Ok(adapter.clone());
    }

    adapters
        .iter()
        .find(|a| a.get_info().backend != wgpu::Backend::Gl)
        .cloned()
        .ok_or_else(|| "No WGPU adapters found".to_owned())
}

fn run() -> Result<(), OmniSttErrors> {
    let settings_manager = SettingsManager::new("omni.toml");
    let settings_general = &settings_manager.settings.general;
    let level = settings_general.level();
    let guard = setup_tracing(level, settings_general.log_to_file());
    let app = SubtitlesApp::new(settings_manager, guard);

    let mut wgpu_configuration = WgpuConfiguration::default();
    if let WgpuSetup::CreateNew(ref mut setup) = wgpu_configuration.wgpu_setup {
        setup.native_adapter_selector = Some(Arc::new(select_adapter));
        setup.instance_descriptor.backends = Backends::PRIMARY | Backends::DX12 | Backends::GL;
    }

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: wgpu_configuration,
        viewport: ViewportBuilder::default()
            .with_app_id("omnistt")
            .with_icon(from_png_bytes(ICON_BYTES).unwrap_or_else(|_| {
                tracing::warn!("Bytes of icon is incorrect...");
                IconData::default()
            }))
            .with_inner_size([400., 600.])
            .with_resizable(false)
            .with_decorations(true)
            .with_always_on_top()
            .with_transparent(true)
            .with_maximize_button(false),
        ..Default::default()
    };

    tracing::info!("Starting application");
    let res = eframe::run_native(
        "OmniStt",
        native_options,
        Box::new(move |cc| {
            setup_custom_fonts(&cc.egui_ctx);
            Ok(Box::new(app))
        }),
    );
    if let Err(e) = res {
        tracing::error!("err: {}", e);
    }

    Ok(())
}

fn main() {
    #[cfg(target_os = "macos")]
    embed_plist::embed_info_plist!("../Info.plist");

    let rt = tokio::runtime::Runtime::new().expect("Should be able to get rt main thread");
    let _e = rt.enter();

    if let Err(err) = run() {
        eprintln!("OmniSTT {:?}", err);
        std::process::exit(1);
    }
}
