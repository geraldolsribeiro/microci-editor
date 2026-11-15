use egui::ScrollArea;
use serde_yaml::{Mapping, Value};

const PADDING: f32 = 5.0;
struct Step {
    data: Mapping,
}

/// The MicroCI editor application.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct MicroCIEditorApp {
    label: String,

    #[serde(skip)]
    steps: Vec<Step>,
}

impl Default for MicroCIEditorApp {
    fn default() -> Self {
        Self {
            label: "MicroCI editor".to_owned(),
            steps: load_steps(&load_yaml()),
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
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if !cfg!(target_arch = "wasm32") {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
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
            ui.label(format!("{} steps", self.steps.len()));
            ui.add_space(PADDING);

            ScrollArea::both().show(ui, |ui| {
                for (index, step) in self.steps.iter().enumerate() {
                    let name = mapping_string(&step.data, "name")
                        .unwrap_or_else(|| format!("Step {}", index + 1));
                    ui.heading(format!("{}. {name}", index + 1));

                    if let Some(description) = mapping_value(&step.data, "description") {
                        ui.label(description_text(description));
                    }

                    ui.add_space(PADDING);
                    for (field_index, (key, value)) in step.data.iter().enumerate() {
                        if key.as_str() == Some("name") || key.as_str() == Some("description") {
                            continue;
                        }
                        // The same field names (for example `plugin`) occur in many steps.
                        // Include the step and field index in the widget ID.
                        ui.push_id((index, field_index), |ui| {
                            ui.collapsing(key_text(key), |ui| show_yaml_value(ui, value));
                        });
                    }
                    ui.add_space(PADDING);
                    ui.separator();
                    ui.add_space(PADDING);
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
        return std::fs::read_to_string(".microCI.yml").unwrap_or_else(|error| {
            let directory = std::env::current_dir()
                .map_or_else(|_| "the current directory".to_owned(), |path| path.display().to_string());
            panic!(
                "Unable to load .microCI.yml from {directory}: {error}. Start the editor from a directory containing .microCI.yml."
            );
        });
    }

    #[cfg(target_arch = "wasm32")]
    panic!("Unable to load .microCI.yml: browser builds cannot read the local filesystem.");
}

fn load_steps(yaml: &str) -> Vec<Step> {
    let document: Value = match serde_yaml::from_str(yaml) {
        Ok(document) => document,
        Err(error) => {
            log::error!("Could not parse src/microCI.yml: {error}");
            return Vec::new();
        }
    };

    document
        .get("steps")
        .and_then(Value::as_sequence)
        .into_iter()
        .flatten()
        .filter_map(Value::as_mapping)
        .cloned()
        .map(|data| Step { data })
        .collect()
}

fn mapping_value<'a>(mapping: &'a Mapping, key: &str) -> Option<&'a Value> {
    mapping.get(Value::String(key.to_owned()))
}

fn mapping_string(mapping: &Mapping, key: &str) -> Option<String> {
    mapping_value(mapping, key)
        .and_then(Value::as_str)
        .map(str::to_owned)
}

fn key_text(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| format!("{value:?}"), str::to_owned)
}

fn description_text(value: &Value) -> String {
    value
        .as_str()
        .map_or_else(|| format!("{value:?}"), str::to_owned)
}

fn show_yaml_value(ui: &mut egui::Ui, value: &Value) {
    match value {
        Value::Mapping(mapping) => {
            for (index, (key, child)) in mapping.iter().enumerate() {
                ui.push_id(index, |ui| {
                    ui.horizontal(|ui| {
                        ui.strong(key_text(key));
                        show_yaml_value(ui, child);
                    });
                });
            }
        }
        Value::Sequence(sequence) => {
            for (index, child) in sequence.iter().enumerate() {
                ui.push_id(index, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("[{index}]"));
                        show_yaml_value(ui, child);
                    });
                });
            }
        }
        _ => {
            ui.label(description_text(value));
        }
    }
}

fn footer(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("microci.dev", "https://microci.dev/");
        ui.label(".");
    });
}
