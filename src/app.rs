use std::str::FromStr as _;

use egui::ScrollArea;
use yaml_edit::{Document, Mapping as YamlMapping};

const PADDING: f32 = 5.0;
#[cfg(target_arch = "wasm32")]
const WEB_DEMO_YAML: &str = include_str!("../.microCI.yml");

/// The `MicroCI` editor application.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MicroCIEditorApp {
    label: String,

    #[serde(skip)]
    document: Document,

    #[serde(skip)]
    status: String,
}

impl Default for MicroCIEditorApp {
    fn default() -> Self {
        Self {
            label: "MicroCI editor".to_owned(),
            document: load_document().expect(".microCI.yml must be a valid YAML document"),
            status: String::new(),
        }
    }
}

impl MicroCIEditorApp {
    pub fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // The YAML is embedded in the binary, so this also works in the web build.
        Self::default()
    }
}

impl eframe::App for MicroCIEditorApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let save_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::S);
        let quit_shortcut = egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::Q);
        if ui.input_mut(|input| input.consume_shortcut(&save_shortcut)) {
            self.status = save_yaml(&self.document);
        } else if ui.input_mut(|input| input.consume_shortcut(&quit_shortcut)) {
            ui.send_viewport_cmd(egui::ViewportCommand::Close);
        }

        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if !cfg!(target_arch = "wasm32") {
                    ui.menu_button("File", |ui| {
                        if ui.button("Save\tCtrl+S").clicked() {
                            self.status = save_yaml(&self.document);
                            ui.close();
                        }
                        if ui.button("Quit\tCtrl+Q").clicked() {
                            ui.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading(&self.label);
            // ui.label("Steps loaded from the YAML document");
            if !self.status.is_empty() {
                ui.label(&self.status);
            }
            ui.add_space(PADDING);

            ScrollArea::both().show(ui, |ui| {
                if let Some(mapping) = self.document.as_mapping()
                    && let Some(steps) = mapping.get_sequence("steps")
                {
                    for (index, step) in (&steps).into_iter().enumerate() {
                        if let Some(step) = step.as_mapping() {
                            let (
                                mut name,
                                mut description,
                                mut only,
                                mut run_as,
                                mut network,
                                mut docker,
                            ) = step_text_fields(step, index + 1);

                            ui.heading(format!("{}. {name}", index + 1));

                            ui.horizontal(|ui| {
                                if ui.text_edit_singleline(&mut name).changed() {
                                    step.set("name", name.clone());
                                }
                                ui.label("name");
                            });

                            ui.horizontal(|ui| {
                                if ui.text_edit_singleline(&mut description).changed() {
                                    step.set("description", description.clone());
                                }
                                ui.label("description");
                            });

                            ui.horizontal(|ui| {
                                if ui.text_edit_singleline(&mut only).changed() {
                                    step.set("only", only.clone());
                                }
                                ui.label("only");
                            });

                            ui.horizontal(|ui| {
                                if ui.text_edit_singleline(&mut run_as).changed() {
                                    step.set("run_as", run_as.clone());
                                }
                                ui.label("run_as");
                            });

                            ui.horizontal(|ui| {
                                if ui.text_edit_singleline(&mut network).changed() {
                                    step.set("network", network.clone());
                                }
                                ui.label("network");
                            });

                            ui.horizontal(|ui| {
                                if ui.text_edit_singleline(&mut docker).changed() {
                                    step.set("docker", docker.clone());
                                }
                                ui.label("docker");
                            });
                        }

                        ui.add_space(PADDING);
                        ui.separator();
                        ui.add_space(PADDING);
                    }
                }

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    footer(ui);
                    egui::warn_if_debug_build(ui);
                });
            });
        });
    }
}

fn load_yaml() -> String {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::fs::read_to_string(".microCI.yml").unwrap_or_else(|error| {
            let directory = std::env::current_dir()
                .map_or_else(|_| "the current directory".to_owned(), |path| path.display().to_string());
            panic!(
                "Unable to load .microCI.yml from {directory}: {error}. Start the editor from a directory containing .microCI.yml."
            );
        })
    }

    #[cfg(target_arch = "wasm32")]
    {
        // The web demo has no access to the user's local filesystem.
        WEB_DEMO_YAML.to_owned()
    }
}

fn save_yaml(document: &Document) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = document;
        return "Saving files is unavailable in the browser build.".to_owned();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        if let Some(root) = document.as_mapping()
            && let Some(steps) = root.get_sequence("steps")
        {
            for step in steps.values() {
                if let Some(step) = step.as_mapping() {
                    let empty_description = step.get("description").is_some_and(|value| {
                        value
                            .as_scalar()
                            .is_some_and(|scalar| scalar.as_string().is_empty())
                    });
                    if empty_description {
                        step.remove("description");
                    } else if step.get("name").is_some()
                        && let Some(description) = step.get("description")
                    {
                        step.move_after("name", "description", description);
                    }
                }
            }
        }

        match std::fs::write(".microCI.yml", document.to_string()) {
            Ok(()) => "Saved .microCI.yml".to_owned(),
            Err(error) => format!("Could not save .microCI.yml: {error}"),
        }
    }
}

fn load_document() -> Option<Document> {
    match Document::from_str(&load_yaml()) {
        Ok(document) => {
            if let Some(root) = document.as_mapping()
                && let Some(steps) = root.get_sequence("steps")
            {
                for step in steps.values() {
                    if let Some(step) = step.as_mapping()
                        && step.get("name").is_some()
                        && let Some(description) = step.get("description")
                    {
                        step.move_after("name", "description", description);
                    }
                }
            }
            Some(document)
        }
        Err(error) => {
            log::error!("Could not parse .microCI.yml: {error}");
            None
        }
    }
}

fn step_text_fields(
    step: &YamlMapping,
    index: usize,
) -> (String, String, String, String, String, String) {
    let field = |key: &str| {
        step.get(key)
            .map_or_else(String::new, |value| yaml_value_text(&value))
    };
    (
        step.get("name")
            .map_or_else(|| format!("Step {index}"), |value| yaml_value_text(&value)),
        field("description"),
        field("only"),
        field("run_as"),
        field("network"),
        field("docker"),
    )
}

fn yaml_value_text(value: &yaml_edit::YamlNode) -> String {
    value
        .as_scalar()
        .map_or_else(|| value.to_string(), |scalar| scalar.as_string())
}

fn footer(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("microci.dev", "https://microci.dev/");
        ui.label(".");
    });
}
