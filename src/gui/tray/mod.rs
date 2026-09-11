pub mod pump;

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
    _icon: TrayIcon,
    ids: [(MenuId, TrayAction); 3],
}

pub struct AppTray {
    events: Receiver<Result<TrayAction, OmniSttErrors>>,
}

impl AppTray {
    pub fn spawn(icon: IconData, ctx: Context) -> Self {
        let (tx, events) = channel();
        std::thread::spawn(move || tray_main(icon, ctx, tx));
        Self { events }
    }

    pub fn poll(&self) -> Option<Result<TrayAction, OmniSttErrors>> {
        self.events.try_recv().ok()
    }
}

fn tray_main(icon: IconData, ctx: Context, tx: Sender<Result<TrayAction, OmniSttErrors>>) {
    let Tray { _icon, ids } = match build_tray(icon) {
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

    Ok(Tray { _icon: icon, ids })
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