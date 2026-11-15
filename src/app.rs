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

    #[serde(skip)]
    status: String,
}

impl Default for MicroCIEditorApp {
    fn default() -> Self {
        Self {
            label: "MicroCI editor".to_owned(),
            steps: load_steps(&load_yaml()),
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
        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                if !cfg!(target_arch = "wasm32") {
                    ui.menu_button("File", |ui| {
                        if ui.button("Save .microCI.yml").clicked() {
                            self.status = save_yaml(&self.steps);
                            ui.close();
                        }
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
            if !self.status.is_empty() {
                ui.label(&self.status);
            }
            ui.add_space(PADDING);

            ScrollArea::both().show(ui, |ui| {
                for (index, step) in self.steps.iter_mut().enumerate() {
                    let name = mapping_string(&step.data, "name")
                        .unwrap_or_else(|| format!("Step {}", index + 1));
                    ui.heading(format!("{}. {name}", index + 1));

                    egui::Grid::new(("step-fields", index))
                        .num_columns(2)
                        .spacing([PADDING * 2.0, PADDING])
                        .show(ui, |ui| {
                            let description_key = Value::String("description".to_owned());
                            let description =
                                step.data.entry(description_key).or_insert_with(|| {
                                    Value::String("no description available".to_owned())
                                });
                            let mut description_text_value = description_text(description);
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                ui.add_sized([90.0, 24.0], egui::Label::new("Description:"));
                            });
                            if ui
                                .add_sized(
                                    [320.0, 24.0],
                                    egui::TextEdit::singleline(&mut description_text_value),
                                )
                                .changed()
                            {
                                *description = Value::String(description_text_value.clone());
                            }
                            ui.end_row();

                            if let Some(only) = step.data.get_mut(Value::String("only".to_owned()))
                            {
                                let mut only_text = only.as_str().unwrap_or_default().to_owned();
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        ui.add_sized([90.0, 24.0], egui::Label::new("Only:"));
                                    },
                                );
                                if ui
                                    .add_sized(
                                        [320.0, 24.0],
                                        egui::TextEdit::singleline(&mut only_text),
                                    )
                                    .changed()
                                {
                                    only_text.retain(|character| character.is_ascii_alphanumeric());
                                    *only = Value::String(only_text);
                                }
                                ui.end_row();
                            }
                        });

                    ui.add_space(PADDING);
                    for (field_index, (key, value)) in step.data.iter().enumerate() {
                        if key.as_str() == Some("name")
                            || key.as_str() == Some("description")
                            || key.as_str() == Some("only")
                        {
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

fn save_yaml(steps: &[Step]) -> String {
    #[cfg(target_arch = "wasm32")]
    {
        let _ = steps;
        return "Saving files is unavailable in the browser build.".to_owned();
    }

    #[cfg(not(target_arch = "wasm32"))]
    {
        let steps = steps
            .iter()
            .map(|step| Value::Mapping(step.data.clone()))
            .collect::<Vec<_>>();
        let mut document = Mapping::new();
        document.insert(Value::String("steps".to_owned()), Value::Sequence(steps));

        match serde_yaml::to_string(&Value::Mapping(document))
            .map_err(|error| error.to_string())
            .and_then(|yaml| std::fs::write(".microCI.yml", yaml).map_err(|error| error.to_string()))
        {
            Ok(()) => "Saved .microCI.yml".to_owned(),
            Err(error) => format!("Could not save .microCI.yml: {error}"),
        }
    }
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
