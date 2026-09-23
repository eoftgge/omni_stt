#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{IconData, ViewportBuilder};
use eframe::egui_wgpu::WgpuSetup;
use eframe::icon_data::from_png_bytes;
use eframe::{WgpuConfiguration, wgpu};
use omni_stt::errors::OmniSttErrors;
use omni_stt::gui::app::SubtitlesApp;
use omni_stt::gui::fonts::setup_custom_fonts;
use omni_stt::gui::theme::apply_theme;
use omni_stt::gui::tray::AppTray;
use omni_stt::logger::setup_tracing;
use omni_stt::settings::{SettingsManager, logging_settings};
use omni_stt::subtitles::transcript;
use omni_stt::{APP_ID, CONFIG_PATH, ICON_BYTES, SETTINGS_WINDOW_SIZE, TOOLTIP};

use std::sync::Arc;

/// Picks an adapter whose surface can actually composite transparency.
///
/// The overlay is a transparent, click-through window. `egui-wgpu` configures
/// the surface with `CompositeAlphaMode::PreMultiplied`, falls back to
/// `PostMultiplied`, and when neither is offered quietly settles for `Auto`.
/// On DX12 and Vulkan that is opaque, and the overlay paints the screen black.
///
/// DX12 and Vulkan report their alpha modes honestly, so an adapter that
/// offers a transparent one is the first choice. The predicate is deliberately
/// the same one `egui-wgpu` applies when it configures the surface.
///
/// GL is the exception: wgpu hard-codes its report to `Opaque` (a TODO in
/// `wgpu-hal`), whatever the driver can do. On Windows its transparency comes
/// from the DWM blur-behind winit sets on the window, not from the swapchain,
/// and it is what keeps the overlay transparent on AMD, where neither DX12 nor
/// Vulkan offers a transparent mode. So when nothing reports one, GL goes next.
fn select_adapter(
    adapters: &[wgpu::Adapter],
    surface: Option<&wgpu::Surface<'_>>,
) -> Result<wgpu::Adapter, String> {
    if let Some(surface) = surface
        && let Some(adapter) = adapters.iter().find(|adapter| {
            surface
                .get_capabilities(adapter)
                .alpha_modes
                .iter()
                .any(|mode| {
                    matches!(
                        mode,
                        wgpu::CompositeAlphaMode::PreMultiplied
                            | wgpu::CompositeAlphaMode::PostMultiplied
                    )
                })
        })
    {
        let info = adapter.get_info();
        tracing::info!(
            "Adapter {} ({:?}) can composite transparency",
            info.name,
            info.backend
        );
        return Ok(adapter.clone());
    }

    tracing::warn!("No adapter reports a transparent surface, preferring GL");
    adapters
        .iter()
        .find(|a| a.get_info().backend == wgpu::Backend::Gl)
        .or_else(|| adapters.first())
        .cloned()
        .ok_or_else(|| "No WGPU adapters found".to_owned())
}

fn run() -> Result<(), OmniSttErrors> {
    let general = logging_settings(CONFIG_PATH);
    let tracing_control = setup_tracing(general.level(), general.log_to_file, general);
    if !transcript::is_local_offset_known() {
        tracing::warn!("Local timezone unavailable, transcript timestamps will be in UTC");
    }
    let settings_manager = SettingsManager::new(CONFIG_PATH);
    let mut wgpu_configuration = WgpuConfiguration::default();
    if let WgpuSetup::CreateNew(ref mut setup) = wgpu_configuration.wgpu_setup {
        setup.native_adapter_selector = Some(Arc::new(select_adapter));
    }
    let icon = from_png_bytes(ICON_BYTES).unwrap_or_else(|_| {
        tracing::warn!("Bytes of icon is incorrect...");
        IconData::default()
    });
    let tray_icon = icon.clone();

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: wgpu_configuration,
        viewport: ViewportBuilder::default()
            .with_app_id(APP_ID)
            .with_icon(icon)
            .with_inner_size(SETTINGS_WINDOW_SIZE)
            .with_resizable(false)
            .with_decorations(true)
            .with_always_on_top()
            .with_transparent(true)
            .with_maximize_button(false),
        ..Default::default()
    };

    tracing::info!("Starting application");
    let res = eframe::run_native(
        TOOLTIP,
        native_options,
        Box::new(move |cc| {
            apply_theme(&cc.egui_ctx);
            setup_custom_fonts(&cc.egui_ctx);
            // See `apply_overlay_window`: hiding clears a screenshot artifact of
            // flip-model presentation, and a GL window never comes back from it.
            let hide_during_restyle = cc
                .wgpu_render_state
                .as_ref()
                .is_some_and(|rs| rs.adapter.get_info().backend != wgpu::Backend::Gl);
            let ctx = cc.egui_ctx.clone();
            let tray = AppTray::spawn(tray_icon, ctx);
            let app =
                SubtitlesApp::new(settings_manager, tracing_control, tray, hide_during_restyle);
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

    // Must precede the runtime: reading the local UTC offset only works while
    // the process is still single-threaded. See `transcript::init_local_offset`.
    transcript::init_local_offset();
    let rt = tokio::runtime::Runtime::new().expect("Should be able to get rt main thread");
    let _e = rt.enter();

    if let Err(err) = run() {
        eprintln!("OmniSTT {:?}", err);
        std::process::exit(1);
    }
}
