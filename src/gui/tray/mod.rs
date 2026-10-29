#[cfg(target_os = "windows")]
pub mod pump;

use std::sync::mpsc::{Receiver, Sender, channel};
#[cfg(target_os = "windows")]
use std::sync::{
    Arc,
    atomic::{AtomicU32, Ordering},
};

use eframe::egui::{Context, IconData};
use tray_icon::menu::{Menu, MenuEvent, MenuId, MenuItem};
use tray_icon::{Icon, TrayIcon, TrayIconBuilder};

use crate::TOOLTIP;
use crate::errors::OmniSttErrors;

#[derive(Clone, Copy, Debug)]
pub enum TrayAction {
    OpenSettings,
    /// Starts the overlay from the settings, or restarts a running one. The
    /// menu item reads "Start" or "Restart" to match.
    Start,
    Quit,
}

fn start_label(running: bool) -> &'static str {
    if running { "Restart" } else { "Start" }
}

struct Tray {
    icon: TrayIcon,
    ids: [(MenuId, TrayAction); 3],
    start: MenuItem,
}

pub struct AppTray {
    events: Receiver<Result<TrayAction, OmniSttErrors>>,
    /// The tray thread's id once its menu exists, 0 before.
    #[cfg(target_os = "windows")]
    tray_thread: Arc<AtomicU32>,
    #[cfg(not(target_os = "windows"))]
    _icon: Option<TrayIcon>,
    #[cfg(not(target_os = "windows"))]
    start_item: Option<MenuItem>,
    /// What the Start item currently says, so it is only changed on a change.
    running: bool,
    tray_failed: bool,
}

impl AppTray {
    #[cfg(target_os = "windows")]
    pub fn spawn(icon: IconData, ctx: Context) -> Self {
        let (tx, events) = channel();
        let tray_thread = Arc::new(AtomicU32::new(0));
        let thread_slot = Arc::clone(&tray_thread);
        std::thread::spawn(move || tray_main(icon, ctx, tx, thread_slot));
        Self {
            events,
            tray_thread,
            running: false,
            tray_failed: false,
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn spawn(icon: IconData, ctx: Context) -> Self {
        let (tx, events) = channel();

        let (icon, start_item) = match build_tray(icon) {
            Ok(Tray { icon, ids, start }) => {
                std::thread::spawn(move || forward_menu_events(ids, ctx, tx));
                (Some(icon), Some(start))
            }
            Err(e) => {
                let _ = tx.send(Err(e));
                (None, None)
            }
        };

        Self {
            events,
            _icon: icon,
            start_item,
            running: false,
            tray_failed: false,
        }
    }

    pub fn poll(&self) -> Option<Result<TrayAction, OmniSttErrors>> {
        self.events.try_recv().ok()
    }

    pub fn set_running(&mut self, running: bool) {
        if self.running == running {
            return;
        }

        #[cfg(target_os = "windows")]
        {
            let thread = self.tray_thread.load(Ordering::Acquire);
            if thread == 0 || !pump::post_running(thread, running) {
                return;
            }
        }
        #[cfg(not(target_os = "windows"))]
        if let Some(item) = &self.start_item {
            item.set_text(start_label(running));
        }

        self.running = running;
    }

    pub fn set_failed(&mut self) {
        self.tray_failed = true;
    }

    pub fn is_failed(&self) -> bool {
        self.tray_failed
    }
}

#[cfg(target_os = "windows")]
fn tray_main(
    icon: IconData,
    ctx: Context,
    tx: Sender<Result<TrayAction, OmniSttErrors>>,
    thread_slot: Arc<AtomicU32>,
) {
    let Tray {
        icon: _icon,
        ids,
        start,
    } = match build_tray(icon) {
        Ok(tray) => tray,
        Err(err) => {
            let _ = tx.send(Err(err));
            ctx.request_repaint();
            return;
        }
    };

    thread_slot.store(pump::current_thread(), Ordering::Release);
    std::thread::spawn(move || forward_menu_events(ids, ctx, tx));
    pump::run(|running| start.set_text(start_label(running)));
}

fn build_tray(icon: IconData) -> Result<Tray, OmniSttErrors> {
    let image = Icon::from_rgba(icon.rgba, icon.width, icon.height)?;

    let open_settings = MenuItem::new("Open Settings", true, None);
    let start = MenuItem::new(start_label(false), true, None);
    let quit = MenuItem::new("Quit", true, None);

    let ids = [
        (open_settings.id().clone(), TrayAction::OpenSettings),
        (start.id().clone(), TrayAction::Start),
        (quit.id().clone(), TrayAction::Quit),
    ];

    let menu = Menu::new();
    menu.append(&open_settings)?;
    menu.append(&start)?;
    menu.append(&quit)?;

    let icon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(TOOLTIP)
        .with_icon(image)
        .build()?;

    Ok(Tray { icon, ids, start })
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
