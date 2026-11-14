use egui::ScrollArea;

#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct CodeEditor {
    language: String,
    code: String,
}

impl Default for CodeEditor {
    fn default() -> Self {
        Self {
            language: "rs".into(),
            code: "// A very simple example\n\
fn main() {\n\
\tprintln!(\"Hello world!\");\n\
}\n\
"
            .into(),
        }
    }
}

struct Step {
    name: String,
    description: String,
    plugin_name: String,
    run_as: String,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TemplateApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    steps: Vec<Step>,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,
}

impl Default for TemplateApp {
    fn default() -> Self {
        let iter = (0..20).map(|i| Step {
            name: format!("Step name {}", i),
            description: format!("Step description {}", i),
            plugin_name: format!("plugin_name_{}", i),
            run_as: format!("run_as{}", i),
        });
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            steps: Vec::from_iter(iter),
            value: 2.7,
        }
    }
}

impl TemplateApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        }
    }
}

impl eframe::App for TemplateApp {
    /// Called by the framework to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::Panel::top("top_panel").show(ui, |ui| {
            // The top panel is often a good place for a menu bar:

            egui::MenuBar::new().ui(ui, |ui| {
                // NOTE: no File->Quit on web pages!
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
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
            // The central panel the region left after adding TopPanel's and SidePanel's
            // ui.heading("MicroCI Editor 0.1.0");
            // ui.separator();

            ScrollArea::both().show(ui, |ui| {
                for s in &self.steps {
                    ui.heading(&s.name);
                    ui.label(&s.description);
                    ui.separator();
                }

                ui.separator();

                ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                    footer(ui);
                    egui::warn_if_debug_build(ui);
                });
            });

            // ui.horizontal(|ui| {
            //     ui.label("xx Write something: ");
            //     ui.text_edit_singleline(&mut self.label);
            // });
            //
            // ui.add(egui::Slider::new(&mut self.value, 0.0..=10.0).text("value"));
            // if ui.button("Increment").clicked() {
            //     self.value += 1.0;
            // }
            //
            // ui.separator();

            // ui.add(egui::github_link_file!(
            //     "https://microci.dev/",
            //     "https://microci.dev/"
            // ));
        });
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
