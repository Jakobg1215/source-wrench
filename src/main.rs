#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod import;
mod input;
mod process;
mod ui;
mod utilities;
mod write;

use eframe::egui;
fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_maximized(true).with_drag_and_drop(false),
        centered: true,
        ..Default::default()
    };
    eframe::run_native("Source Wrench", options, Box::new(|_| Ok(Box::<SourceWrenchApplication>::default())))
}

use egui_dock::DockState;
use import::{FileManager, FileStatus, SUPPORTED_FILES};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
struct SourceWrenchApplication {
    tab_tree: DockState<SourceWrenchTabType>,
    compiling: Arc<AtomicBool>,
    input_data: InputCompilationData,
    loaded_files: FileManager,
}

impl Default for SourceWrenchApplication {
    fn default() -> Self {
        let mut tree = DockState::new(vec![SourceWrenchTabType::Main]);

        let [main_tab, logging_tab] = tree
            .main_surface_mut()
            .split_right(egui_dock::NodeIndex::root(), 0.5, vec![SourceWrenchTabType::Logging]);

        let [_, _] = tree.main_surface_mut().split_below(main_tab, 0.35, vec![SourceWrenchTabType::BodyGroups]);

        let [_, _] = tree
            .main_surface_mut()
            .split_below(logging_tab, 0.35, vec![SourceWrenchTabType::Animations, SourceWrenchTabType::Sequences]);

        let mut loaded_files = FileManager::default();

        if let Err(watch_error) = loaded_files.start_file_watch() {
            error!("Fail To Start File Watch: {watch_error}!");
        }

        Self {
            tab_tree: tree,
            compiling: Arc::new(AtomicBool::new(false)),
            input_data: Default::default(),
            loaded_files,
        }
    }
}

impl eframe::App for SourceWrenchApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut new_tabs = Vec::new();

        egui_dock::DockArea::new(&mut self.tab_tree)
            .style(egui_dock::Style::from_egui(ctx.style().as_ref()))
            .show_leaf_close_all_buttons(false)
            .show_add_buttons(true)
            .show_add_popup(true)
            .show(
                ctx,
                &mut SourceWrenchTabManager {
                    new_tabs: &mut new_tabs,
                    compiling: Arc::clone(&self.compiling),
                    input_data: &mut self.input_data,
                    loaded_files: &mut self.loaded_files,
                },
            );

        new_tabs.drain(..).for_each(|tab| {
            let existing_tab = self.tab_tree.find_tab(&tab);

            if let Some(existing_tab) = existing_tab {
                self.tab_tree.set_active_tab(existing_tab);
                return;
            }

            self.tab_tree.push_to_focused_leaf(tab);
        });
    }
}

#[derive(PartialEq)]
enum SourceWrenchTabType {
    Main,
    Logging,
    BodyGroups,
    Animations,
    Sequences,
}

impl SourceWrenchTabType {
    fn all_closeable_variants() -> Vec<Self> {
        vec![Self::BodyGroups, Self::Animations, Self::Sequences]
    }
}

struct SourceWrenchTabManager<'a> {
    new_tabs: &'a mut Vec<SourceWrenchTabType>,
    compiling: Arc<AtomicBool>,
    input_data: &'a mut InputCompilationData,
    loaded_files: &'a mut FileManager,
}

impl egui_dock::TabViewer for SourceWrenchTabManager<'_> {
    type Tab = SourceWrenchTabType;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match &tab {
            SourceWrenchTabType::Main => String::from("Main").into(),
            SourceWrenchTabType::Logging => String::from("Log").into(),
            SourceWrenchTabType::BodyGroups => String::from("Body Groups").into(),
            SourceWrenchTabType::Animations => String::from("Animations").into(),
            SourceWrenchTabType::Sequences => String::from("Sequences").into(),
        }
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            SourceWrenchTabType::Main => self.render_main(ui),
            SourceWrenchTabType::Logging => self.render_logging(ui),
            SourceWrenchTabType::BodyGroups => self.render_body_groups(ui),
            SourceWrenchTabType::Animations => self.render_animations(ui),
            SourceWrenchTabType::Sequences => self.render_sequences(ui),
        }
    }

    fn scroll_bars(&self, _tab: &Self::Tab) -> [bool; 2] {
        [false, false]
    }

    fn closeable(&mut self, tab: &mut Self::Tab) -> bool {
        !matches!(tab, SourceWrenchTabType::Main | SourceWrenchTabType::Logging)
    }

    fn add_popup(&mut self, ui: &mut egui::Ui, _surface: egui_dock::SurfaceIndex, _node: egui_dock::NodeIndex) {
        ui.set_min_width(100.0);
        ui.style_mut().visuals.button_frame = false;

        // TODO: Only show the ones that are missing.
        for mut tab in SourceWrenchTabType::all_closeable_variants() {
            if ui.button(self.title(&mut tab)).clicked() {
                self.new_tabs.push(tab);
            }
        }
    }
}

use input::InputCompilationData;
use ui::{icon, toggle_ui_compact};
use utilities::logging;

use crate::ui::ListSelect;
impl SourceWrenchTabManager<'_> {
    fn render_main(&mut self, ui: &mut egui::Ui) {
        ui.heading("Source Wrench");
        ui.label("A Source Engine Model Compiler");

        ui.separator();

        let name_label = ui.label("Model Out Directory: ");
        if ui
            .text_edit_singleline(
                &mut self
                    .input_data
                    .export_path
                    .as_ref()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|| String::from("Select a directory...")),
            )
            .labelled_by(name_label.id)
            .clicked()
        {
            if let Some(path) = rfd::FileDialog::new().set_title("Select Export Path").pick_folder() {
                self.input_data.export_path = Some(path);
            }
        }

        if let Some(export_path) = &self.input_data.export_path {
            let name_label = ui.label("Model Name: ");
            ui.text_edit_singleline(&mut self.input_data.model_name).labelled_by(name_label.id);

            let is_compiling = self.compiling.load(std::sync::atomic::Ordering::Relaxed);

            let button_response = ui.add_enabled(!is_compiling, egui::Button::new("Compile Model"));
            if button_response.clicked() {
                // The best thing to do is just to clone the data.
                let input_data = self.input_data.clone();
                let loaded_files = self.loaded_files.clone();
                let export_path = export_path.to_string_lossy().to_string();
                let compiling = Arc::clone(&self.compiling);
                compiling.store(true, std::sync::atomic::Ordering::Relaxed);

                std::thread::spawn(move || {
                    if input_data.model_name.is_empty() {
                        error!("Model name is empty!");
                        compiling.store(false, Ordering::Relaxed);
                        return;
                    }

                    let mut model_name = input_data.model_name.clone();
                    if !model_name.ends_with(".mdl") {
                        model_name.push_str(".mdl");
                    }

                    info!("Processing {}!", &model_name);

                    let processed_data = match process::process(&input_data, &loaded_files) {
                        Ok(data) => data,
                        Err(error) => {
                            error!("Fail To Compile Model: {error}!");
                            compiling.store(false, Ordering::Relaxed);
                            return;
                        }
                    };

                    info!("Writing Files!");

                    match write::write_files(input_data.model_name.clone(), model_name, processed_data, export_path) {
                        Ok(_) => {}
                        Err(error) => {
                            error!("Fail To Write Files: {error}!");
                            compiling.store(false, Ordering::Relaxed);
                            return;
                        }
                    }

                    info!("Model compiled successfully!");
                    compiling.store(false, Ordering::Relaxed);
                });
            }
        }
    }

    fn render_logging(&mut self, ui: &mut egui::Ui) {
        let mut logger = logging::LOGGER.lock();
        ui.horizontal(|ui| {
            ui.horizontal(|ui| {
                ui.label("Verbose:");
                toggle_ui_compact(ui, &mut logger.allow_verbose);

                ui.label("Debug:");
                toggle_ui_compact(ui, &mut logger.allow_debug);
            });

            if ui.button("Clear Log").clicked() {
                logger.logs.clear();
            }
        });

        ui.separator();

        egui::ScrollArea::vertical().auto_shrink([false; 2]).stick_to_bottom(true).show(ui, |ui| {
            for (log, level) in &logger.logs {
                let log_color = match level {
                    logging::LogLevel::Info => egui::Color32::DARK_GREEN,
                    logging::LogLevel::Verbose => egui::Color32::MAGENTA,
                    logging::LogLevel::Debug => egui::Color32::CYAN,
                    logging::LogLevel::Warn => egui::Color32::YELLOW,
                    logging::LogLevel::Error => egui::Color32::RED,
                };

                if matches!(level, logging::LogLevel::Verbose) && !logger.allow_verbose {
                    continue;
                }

                if matches!(level, logging::LogLevel::Debug) && !logger.allow_debug {
                    continue;
                }

                ui.colored_label(log_color, log);
                ui.separator();
            }
        });
    }

    fn render_body_groups(&mut self, ui: &mut egui::Ui) {
        ui.heading("Body Groups");
        let selected_body_group = ListSelect::new("Body Group List").show(ui, &mut self.input_data.body_groups, |body_group| &mut body_group.name);
        ui.separator();
        if let Some(active_body_group) = selected_body_group {
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                let name_label = ui.label("Body Group Name: ");
                ui.text_edit_singleline(&mut active_body_group.name).labelled_by(name_label.id);

                egui::CollapsingHeader::new("Models").default_open(true).show(ui, |ui| {
                    let selected_model = ListSelect::new("Model List").show(ui, &mut active_body_group.models, |model| &mut model.name);
                    ui.separator();

                    if let Some(active_model) = selected_model {
                        ui.checkbox(&mut active_model.blank, "Blank");

                        if active_model.blank {
                            return;
                        }

                        let name_label = ui.label("Model Name: ");
                        ui.text_edit_singleline(&mut active_model.name).labelled_by(name_label.id);
                        if ui.button("Select Model File…").clicked() {
                            if let Some(path) = rfd::FileDialog::new()
                                .set_title("Select Model File")
                                .add_filter("Supported Files", &SUPPORTED_FILES)
                                .pick_file()
                            {
                                if let Some(last_path) = &active_model.source_file_path {
                                    self.loaded_files.unload_file(last_path);
                                };
                                active_model.source_file_path = Some(path.clone());
                                self.loaded_files.load_file(path);
                            }
                        }

                        if let Some(source_file_path) = &active_model.source_file_path {
                            let file_status = self.loaded_files.get_file_status(source_file_path).unwrap(); // FIXME: This must be check!

                            ui.horizontal(|ui| {
                                ui.label("Model File:");
                                ui.monospace(source_file_path.display().to_string());
                                match file_status {
                                    FileStatus::Loading => {
                                        ui.spinner();
                                        if !active_model.enabled_source_parts.is_empty() {
                                            active_model.enabled_source_parts.clear();
                                        }
                                    }
                                    FileStatus::Loaded(_) => {
                                        ui.add(icon(ui::IconType::Check));
                                    }
                                    FileStatus::Failed => {
                                        ui.add(icon(ui::IconType::X));
                                        if !active_model.enabled_source_parts.is_empty() {
                                            active_model.enabled_source_parts.clear();
                                        }
                                    }
                                }
                            });

                            if let FileStatus::Loaded(file_data) = file_status {
                                if file_data.parts.is_empty() {
                                    ui.colored_label(egui::Color32::RED, "Model File Has No Mesh!");
                                    return;
                                }

                                if active_model.enabled_source_parts.is_empty() {
                                    active_model.enabled_source_parts = vec![true; file_data.parts.len()];
                                }

                                ui.heading("Enabled Parts");
                                ui.separator();
                                egui::ScrollArea::horizontal().show(ui, |ui| {
                                    for (part_index, (part_name, _)) in file_data.parts.iter().enumerate() {
                                        let enabled_source_part = &mut active_model.enabled_source_parts[part_index];
                                        ui.checkbox(enabled_source_part, part_name);
                                    }
                                });
                            }
                        }
                    }
                });
            });
        }
    }

    fn render_animations(&mut self, ui: &mut egui::Ui) {
        ui.heading("Animations");
        let selected_animation = ListSelect::new("Animation List").show(ui, &mut self.input_data.animations, |animation| &mut animation.name);
        ui.separator();
        if let Some(active_animation) = selected_animation {
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                let name_label = ui.label("Animation Name: ");
                ui.text_edit_singleline(&mut active_animation.name).labelled_by(name_label.id);

                if ui.button("Select Model File…").clicked() {
                    if let Some(path) = rfd::FileDialog::new()
                        .set_title("Select Model File")
                        .add_filter("Supported Files", &SUPPORTED_FILES)
                        .pick_file()
                    {
                        if let Some(last_path) = &active_animation.source_file_path {
                            self.loaded_files.unload_file(last_path);
                        };
                        active_animation.source_file_path = Some(path.clone());
                        self.loaded_files.load_file(path);
                    }
                }

                if let Some(source_file_path) = &active_animation.source_file_path {
                    let file_status = self.loaded_files.get_file_status(source_file_path).unwrap(); // FIXME: This must be check!

                    ui.horizontal(|ui| {
                        ui.label("Animation File:");
                        ui.monospace(source_file_path.display().to_string());
                        match file_status {
                            FileStatus::Loading => {
                                ui.spinner();
                                active_animation.source_animation = 0;
                            }
                            FileStatus::Loaded(_) => {
                                ui.add(icon(ui::IconType::Check));
                            }
                            FileStatus::Failed => {
                                ui.add(icon(ui::IconType::X));
                                active_animation.source_animation = 0;
                            }
                        }
                    });

                    if let FileStatus::Loaded(file_data) = file_status {
                        if active_animation.source_animation > file_data.animations.len() {
                            active_animation.source_animation = 0;
                        }

                        ui.separator();
                        egui::ComboBox::from_label("Source Animation")
                            .selected_text(file_data.animations.get_index(active_animation.source_animation).unwrap().0)
                            .show_ui(ui, |ui| {
                                for (source_animation_index, (source_animation_name, _)) in file_data.animations.iter().enumerate() {
                                    ui.selectable_value(&mut active_animation.source_animation, source_animation_index, source_animation_name);
                                }
                            });
                    }
                }
            });
        }
    }

    fn render_sequences(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sequences");
        let selected_sequence = ListSelect::new("Sequence List").show(ui, &mut self.input_data.sequences, |sequence| &mut sequence.name);
        ui.separator();
        if let Some(active_sequence) = selected_sequence {
            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                let name_label = ui.label("Sequence Name: ");
                ui.text_edit_singleline(&mut active_sequence.name).labelled_by(name_label.id);

                if self.input_data.animations.is_empty() {
                    ui.colored_label(egui::Color32::RED, "No Animations Created");

                    if !active_sequence.animations.is_empty() {
                        active_sequence.animations.clear();
                    }

                    return;
                }

                if active_sequence.animations.is_empty() {
                    active_sequence.animations = vec![vec![0]];
                }

                let sequence_animation = &mut active_sequence.animations[0][0];
                egui::ComboBox::from_label("Selected Animation")
                    .selected_text(&self.input_data.animations[*sequence_animation].name)
                    .show_ui(ui, |ui| {
                        for (animation_index, animation) in self.input_data.animations.iter().enumerate() {
                            ui.selectable_value(sequence_animation, animation_index, &animation.name);
                        }
                    });
            });
        }
    }
}
