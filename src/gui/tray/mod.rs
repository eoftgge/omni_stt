#[cfg(target_os = "windows")] pub mod pump;

use std::sync::mpsc::{Receiver, Sender, channel};

use eframe::egui::{Context, IconData};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::errors::OmniSttErrors;
use crate::TOOLTIP;

#[derive(Clone, Copy, Debug)]
pub enum TrayAction {
    OpenSettings,
    Restart,
    Quit,
}

struct Tray {
    icon: TrayIcon,
    ids: [(MenuId, TrayAction); 3],
}

pub struct AppTray {
    events: Receiver<Result<TrayAction, OmniSttErrors>>,
    #[cfg(not(target_os = "windows"))]
    _icon: Option<TrayIcon>,
}

impl AppTray {
    #[cfg(target_os = "windows")]
    pub fn spawn(icon: IconData, ctx: Context) -> Self {
        let (tx, events) = channel();
        std::thread::spawn(move || tray_main(icon, ctx, tx));
        Self { events }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn spawn(icon: IconData, ctx: Context) -> Self {
        let (tx, events) = channel();

        let icon = match build_tray(icon) {
            Ok(Tray { icon, ids }) => {
                std::thread::spawn(move || forward_menu_events(ids, ctx, tx));
                Some(icon)
            }
            Err(e) => {
                let _ = tx.send(Err(e));
                None
            }
        };

        Self { events, _icon: icon }
    }

    pub fn poll(&self) -> Option<Result<TrayAction, OmniSttErrors>> {
        self.events.try_recv().ok()
    }
}

#[cfg(target_os = "windows")]
fn tray_main(icon: IconData, ctx: Context, tx: Sender<Result<TrayAction, OmniSttErrors>>) {
    let Tray { icon: _icon, ids } = match build_tray(icon) {
        Ok(tray) => tray,
        Err(err) => {
            let _ = tx.send(Err(err));
            ctx.request_repaint();
            return;
        }
    };

    std::thread::spawn(move || forward_menu_events(ids, ctx, tx));
    pump::run();
}

fn build_tray(icon: IconData) -> Result<Tray, OmniSttErrors> {
    let image = Icon::from_rgba(icon.rgba, icon.width, icon.height)?;

    let open_settings = MenuItem::new("Open Settings", true, None);
    let restart = MenuItem::new("Restart", true, None);
    let quit = MenuItem::new("Quit", true, None);

    let ids = [
        (open_settings.id().clone(), TrayAction::OpenSettings),
        (restart.id().clone(), TrayAction::Restart),
        (quit.id().clone(), TrayAction::Quit),
    ];

    let menu = Menu::new();
    menu.append(&open_settings)?;
    menu.append(&restart)?;
    menu.append(&quit)?;

    let icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(TOOLTIP)
        .with_icon(image)
        .build()?;

    Ok(Tray { icon, ids })
}

fn forward_menu_events(
    ids: [(MenuId, TrayAction); 3],
    ctx: Context,
    tx: Sender<Result<TrayAction, OmniSttErrors>>,
) {
    while let Ok(event) = MenuEvent::receiver().recv() {
        let Some((_, action)) = ids.iter().find(|(id, _)| *id == event.id) else {
            continue;
        };

        if tx.send(Ok(*action)).is_err() {
            break;
        }
        ctx.request_repaint();
    }
}