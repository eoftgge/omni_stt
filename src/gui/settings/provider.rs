use super::layout::{Squared, label, row, section, settings_grid};
use crate::settings::provider::SettingsProvider;
use crate::settings::KeyStorage;
use crate::stt::adapters::types::{ProviderType, SonioxSettings, VoskSettings};
use crate::stt::adapters::vosk::probe::VoskProbe;
use crate::stt::languages::LanguageHint;
use eframe::egui::{self, Checkbox, Color32, ComboBox, RichText, TextEdit, Ui};
use std::fmt::Debug;
use std::path::{Path, PathBuf};

pub(super) fn ui_section_provider(
    ui: &mut Ui,
    settings_provider: &mut SettingsProvider,
    key_storage: &KeyStorage,
    vosk_probe: &mut VoskProbe,
) {
    section(ui, "Speech Engine (STT)", true, |ui| {
        ui.horizontal(|ui| {
            ui.selectable_value(
                &mut settings_provider.active_type,
                ProviderType::Soniox,
                "☁ Soniox (Cloud)",
            );
            ui.selectable_value(
                &mut settings_provider.active_type,
                ProviderType::Vosk,
                "💻 Vosk (Offline)",
            );
        });

        ui.separator();

        match settings_provider.active_type {
            ProviderType::Soniox => {
                ui_soniox_settings(ui, &mut settings_provider.soniox, key_storage)
            }
            ProviderType::Vosk => ui_vosk_settings(ui, &mut settings_provider.vosk, vosk_probe),
        }
    });
}

fn ui_soniox_settings(ui: &mut Ui, soniox: &mut SonioxSettings, key_storage: &KeyStorage) {
    settings_grid("soniox_grid").show(ui, |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            label(ui, "API Key:");
        });
        ui.vertical(|ui| {
            ui.add(TextEdit::singleline(&mut soniox.api_key.0).password(true));
            ui_key_storage_hint(ui, key_storage);
        });
        ui.end_row();

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            label(ui, "Languages:");
        });
        ui.vertical(|ui| {
            let mut to_remove = None;
            for (i, hint) in soniox.language_hints.iter_mut().enumerate() {
                ui.horizontal(|ui| {
                    label(ui, &format!("{}.", i + 1));
                    ui_language_searchable_combo(ui, format!("hint_{}", i), hint);
                    if ui.button("🗑").clicked() {
                        to_remove = Some(i);
                    }
                });
            }
            if let Some(i) = to_remove {
                soniox.language_hints.remove(i);
            }
            if ui.button("➕ Add").clicked() {
                soniox.language_hints.push(LanguageHint::English);
            }
        });
        ui.end_row();

        label(ui, "Translation:");
        ui.add(Squared(Checkbox::new(
            &mut soniox.enable_translate,
            "Enable",
        )));

        if soniox.enable_translate {
            ui.end_row();
            ui.add(egui::Label::new("Target language:").extend());
            ui_language_searchable_combo(ui, "target_lang", &mut soniox.target_language);
        }
        ui.end_row();

        row(
            ui,
            "Context:",
            TextEdit::multiline(&mut soniox.context).desired_rows(2),
        );
        row(
            ui,
            "Options:",
            Squared(Checkbox::new(
                &mut soniox.enable_speakers,
                "Enable Speakers ID",
            )),
        );
    });
}

fn ui_vosk_settings(ui: &mut Ui, vosk: &mut VoskSettings, probe: &mut VoskProbe) {
    settings_grid("vosk_grid").show(ui, |ui| {
        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            label(ui, "Model:");
        });
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let mut path = vosk.model_path.display().to_string();
                if ui
                    .add(TextEdit::singleline(&mut path).desired_width(200.0))
                    .changed()
                {
                    vosk.model_path = PathBuf::from(path);
                }
                if ui
                    .button("📂")
                    .on_hover_text("Pick the model folder")
                    .clicked()
                    && let Some(picked) = rfd::FileDialog::new().pick_folder()
                {
                    vosk.model_path = picked;
                }
            });
            ui_model_hint(ui, &vosk.model_path);
        });
        ui.end_row();

        ui.with_layout(egui::Layout::top_down(egui::Align::Min), |ui| {
            label(ui, "Library:");
        });
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                let mut path = vosk.library_path.display().to_string();
                if ui
                    .add(
                        TextEdit::singleline(&mut path)
                            .desired_width(200.0)
                            .hint_text("auto"),
                    )
                    .changed()
                {
                    vosk.library_path = PathBuf::from(path);
                }
                if ui.button("📂").on_hover_text("Pick libvosk").clicked()
                    && let Some(picked) = rfd::FileDialog::new().pick_file()
                {
                    vosk.library_path = picked;
                }
            });
            ui_library_hint(ui, probe.check(&vosk.library_path));
        });
        ui.end_row();
    });
}

fn ui_language_searchable_combo(
    ui: &mut Ui,
    id_salt: impl std::hash::Hash + Debug,
    selected: &mut LanguageHint,
) {
    let id = ui.make_persistent_id(id_salt);
    let mut search_term = ui.data_mut(|d| d.get_temp::<String>(id).unwrap_or_default());

    ComboBox::from_id_salt(id)
        .selected_text(selected.to_string())
        .height(250.)
        .show_ui(ui, |ui| {
            ui.set_min_width(180.0);
            ui.set_min_height(250.0);
            let text_edit_response = ui.add(
                TextEdit::singleline(&mut search_term)
                    .hint_text("🔍 Search...")
                    .desired_width(f32::INFINITY),
            );

            if !text_edit_response.has_focus() {
                text_edit_response.request_focus();
            }

            ui.separator();
            let query = search_term.to_lowercase();
            for lang in LanguageHint::all() {
                let lang_name = lang.to_string();
                if (query.is_empty() || lang_name.to_lowercase().contains(&query))
                    && ui.selectable_value(selected, *lang, lang_name).clicked()
                {
                    search_term.clear();
                }
            }
        });

    ui.data_mut(|d| d.insert_temp(id, search_term));
}

fn ui_key_storage_hint(ui: &mut Ui, key_storage: &KeyStorage) {
    match key_storage {
        KeyStorage::Keyring => {
            ui.label(
                RichText::new("🔒 Stored in the system keychain")
                    .small()
                    .weak(),
            );
        }
        KeyStorage::PlainFile { reason } => {
            ui.label(
                RichText::new(
                    "⚠ System keychain unavailable — the key is stored in omni.toml as plain text",
                )
                .small()
                .color(Color32::from_rgb(220, 160, 60)),
            )
            .on_hover_text(reason);
        }
    }
}

fn ui_model_hint(ui: &mut Ui, path: &Path) {
    if path.as_os_str().is_empty() {
        ui.label(
            RichText::new("Pick the folder of an unpacked Vosk model")
                .small()
                .weak(),
        );
    } else if path.join("am").is_dir() && path.join("conf").is_dir() {
        ui.label(RichText::new("✔ Looks like a Vosk model").small().weak());
    } else {
        ui.label(
            RichText::new("⚠ No am/ and conf/ inside — probably not a model folder")
                .small()
                .color(Color32::from_rgb(220, 160, 60)),
        );
    }
}

fn ui_library_hint(ui: &mut Ui, status: &Result<(), String>) {
    match status {
        Ok(()) => {
            ui.label(RichText::new("✔ libvosk found").small().weak());
        }
        Err(reason) => {
            ui.label(
                RichText::new("⚠ libvosk not found")
                    .small()
                    .color(Color32::from_rgb(220, 160, 60)),
            )
            .on_hover_text(reason);
            ui.hyperlink_to(
                RichText::new("Get it from the Vosk releases").small(),
                "https://github.com/alphacep/vosk-api/releases",
            );
        }
    }
}
