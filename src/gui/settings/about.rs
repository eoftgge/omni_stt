use super::layout::{row, section, settings_grid};
use eframe::egui::{Hyperlink, RichText, Ui};

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");

/// Licence and notices link to the release tag, not `main`: they must describe
/// the build the user is running, and `main` may have moved on since.
pub(super) fn ui_section_about(ui: &mut Ui) {
    section(ui, "About", false, |ui| {
        ui.label(RichText::new(format!("Omni-STT {VERSION}")).strong());
        ui.label("Real-time subtitles over everything, from your speakers or your microphone.");

        settings_grid("about_grid").show(ui, |ui| {
            row(
                ui,
                "Source:",
                Hyperlink::from_label_and_url("GitHub", REPOSITORY),
            );
            row(
                ui,
                "Report a bug:",
                Hyperlink::from_label_and_url("Issues", format!("{REPOSITORY}/issues")),
            );
            row(
                ui,
                "License:",
                Hyperlink::from_label_and_url(
                    "MIT",
                    format!("{REPOSITORY}/blob/v{VERSION}/LICENSE"),
                ),
            );
            row(
                ui,
                "Third-party:",
                Hyperlink::from_label_and_url(
                    "Notices",
                    format!("{REPOSITORY}/blob/v{VERSION}/THIRD-PARTY-NOTICES.md"),
                ),
            );
        });
    });
}
