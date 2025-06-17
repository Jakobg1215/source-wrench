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
        viewport: egui::ViewportBuilder::default().with_inner_size([1440.0, 720.0]).with_drag_and_drop(false),
        centered: true,
        ..Default::default()
    };
    eframe::run_native("Source Wrench", options, Box::new(|_| Ok(Box::<SourceWrenchApplication>::default())))
}

use egui_dock::DockState;
use import::{FileManager, FileStatus, SUPPORTED_FILES};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
struct SourceWrenchApplication {
    tab_tree: DockState<SourceWrenchTabType>,
    compiling: Arc<AtomicBool>,
    input_data_identifier_generator: usize,
    input_data: ImputedCompilationData,
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
            log(format!("Fail To Start File Watch: {}!", watch_error), LogLevel::Error);
        }

        Self {
            tab_tree: tree,
            compiling: Arc::new(AtomicBool::new(false)),
            input_data_identifier_generator: Default::default(),
            input_data: Default::default(),
            loaded_files,
        }
    }
}

impl eframe::App for SourceWrenchApplication {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut new_tabs = Vec::new();

        ctx.set_pixels_per_point(1.25);

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
                    input_data_identifier_generator: &mut self.input_data_identifier_generator,
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
    input_data_identifier_generator: &'a mut usize,
    input_data: &'a mut ImputedCompilationData,
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

use input::{ImputedAnimation, ImputedBodyPart, ImputedCompilationData, ImputedModel, ImputedSequence};
use ui::{toggle_ui_compact, ui_failed, ui_success};
use utilities::logging::{self, log, LogLevel};
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
                        log("Model name is empty!", LogLevel::Error);
                        compiling.store(false, Ordering::Relaxed);
                        return;
                    }

                    let mut model_name = input_data.model_name.clone();
                    if !model_name.ends_with(".mdl") {
                        model_name.push_str(".mdl");
                    }

                    log(format!("Processing {}!", &model_name), LogLevel::Info);

                    let processed_data = match process::process(&input_data, &loaded_files) {
                        Ok(data) => data,
                        Err(error) => {
                            log(format!("Fail To Compile Model: {}!", error), LogLevel::Error);
                            compiling.store(false, Ordering::Relaxed);
                            return;
                        }
                    };

                    log("Writing Files!", LogLevel::Info);

                    match write::write_files(input_data.model_name.clone(), model_name, processed_data, export_path) {
                        Ok(_) => {}
                        Err(error) => {
                            log(format!("Fail To Write Files: {}!", error), LogLevel::Error);
                            compiling.store(false, Ordering::Relaxed);
                            return;
                        }
                    }

                    log("Model compiled successfully!", LogLevel::Info);
                    compiling.store(false, Ordering::Relaxed);
                });
            }
        }
    }

    fn render_logging(&mut self, ui: &mut egui::Ui) {
        let mut logger = logging::LOGGER.lock().unwrap();
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
                    logging::LogLevel::Log => egui::Color32::GRAY,
                    logging::LogLevel::Info => egui::Color32::DARK_GREEN,
                    logging::LogLevel::Verbose => egui::Color32::MAGENTA,
                    logging::LogLevel::Debug => egui::Color32::CYAN,
                    logging::LogLevel::Warn => egui::Color32::YELLOW,
                    logging::LogLevel::Error => egui::Color32::RED,
                };

                if matches!(level, LogLevel::Verbose) && !logger.allow_verbose {
                    continue;
                }

                if matches!(level, LogLevel::Debug) && !logger.allow_debug {
                    continue;
                }

                ui.colored_label(log_color, log);
                ui.separator();
            }
        });
    }

    fn render_body_groups(&mut self, ui: &mut egui::Ui) {
        ui.heading("Body Groups");
        if ui.button("Add Body Group").clicked() {
            self.input_data
                .body_groups
                .insert(*self.input_data_identifier_generator, ImputedBodyPart::default());
            *self.input_data_identifier_generator += 1;
        }

        ui.separator();

        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            let mut removed_body_groups = Vec::new();
            for body_group_index in 0..self.input_data.body_groups.len() {
                let (body_group_identifier, body_group) = &mut self.input_data.body_groups.get_index_mut(body_group_index).unwrap();
                egui::CollapsingHeader::new(format!("Body Group: {}", body_group.name))
                    .id_salt(**body_group_identifier)
                    .default_open(true)
                    .show(ui, |ui| {
                        let name_label = ui.label("Body Group Name: ");
                        ui.text_edit_singleline(&mut body_group.name).labelled_by(name_label.id);

                        ui.horizontal(|ui| {
                            if ui.button("Add Model").clicked() {
                                body_group.models.insert(*self.input_data_identifier_generator, ImputedModel::default());
                                *self.input_data_identifier_generator += 1;
                            }

                            if ui.button("Remove Body Group").clicked() {
                                removed_body_groups.push(**body_group_identifier);
                            }
                        });

                        let mut removed_models = Vec::new();
                        for model_index in 0..body_group.models.len() {
                            let (model_identifier, model) = &mut body_group.models.get_index_mut(model_index).unwrap();
                            egui::CollapsingHeader::new(format!("Model: {}", model.name))
                                .default_open(true)
                                .id_salt(**model_identifier)
                                .show(ui, |ui| {
                                    ui.checkbox(&mut model.blank, "Blank");

                                    if model.blank {
                                        return;
                                    }

                                    let name_label = ui.label("Model Name: ");
                                    ui.text_edit_singleline(&mut model.name).labelled_by(name_label.id);
                                    ui.horizontal(|ui| {
                                        if ui.button("Select Model File…").clicked() {
                                            if let Some(path) = rfd::FileDialog::new()
                                                .set_title("Select Model File")
                                                .add_filter("Supported Files", &SUPPORTED_FILES)
                                                .pick_file()
                                            {
                                                if let Some(last_path) = &model.source_file_path {
                                                    self.loaded_files.unload_file(last_path);
                                                };
                                                model.source_file_path = Some(path.clone());
                                                self.loaded_files.load_file(path);
                                            }
                                        }

                                        if ui.button("Remove Model").clicked() {
                                            removed_models.push(**model_identifier);
                                        }
                                    });

                                    if let Some(source_file_path) = &model.source_file_path {
                                        let file_status = self.loaded_files.get_file_status(source_file_path).unwrap(); // FIXME: This must be check!

                                        ui.horizontal(|ui| {
                                            ui.label("Model File:");
                                            ui.monospace(source_file_path.display().to_string());
                                            match file_status {
                                                FileStatus::Loading => {
                                                    ui.spinner();
                                                    if !model.enabled_source_parts.is_empty() {
                                                        model.enabled_source_parts.clear();
                                                    }
                                                }
                                                FileStatus::Loaded(_) => {
                                                    ui_success(ui);
                                                }
                                                FileStatus::Failed => {
                                                    ui_failed(ui);
                                                    if !model.enabled_source_parts.is_empty() {
                                                        model.enabled_source_parts.clear();
                                                    }
                                                }
                                            }
                                        });

                                        if let FileStatus::Loaded(file_data) = file_status {
                                            if file_data.parts.is_empty() {
                                                ui.colored_label(egui::Color32::RED, "Model File Has No Mesh!");
                                                return;
                                            }

                                            if model.enabled_source_parts.is_empty() {
                                                model.enabled_source_parts = vec![true; file_data.parts.len()];
                                            }

                                            ui.heading("Enabled Parts");
                                            ui.separator();
                                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                                for (part_index, (part_name, _)) in file_data.parts.iter().enumerate() {
                                                    let enabled_source_part = &mut model.enabled_source_parts[part_index];
                                                    ui.checkbox(enabled_source_part, part_name);
                                                }
                                            });
                                        }
                                    }
                                });
                        }

                        for removed_model in removed_models {
                            let removed_model = body_group.models.shift_remove(&removed_model);
                            if let Some(model) = removed_model {
                                if let Some(path) = model.source_file_path {
                                    self.loaded_files.unload_file(&path);
                                }
                            }
                        }
                    });
            }

            for removed_body_group in removed_body_groups {
                let removed_body_part = self.input_data.body_groups.shift_remove(&removed_body_group);
                if let Some(body_part) = removed_body_part {
                    for (_, model) in body_part.models {
                        if let Some(path) = model.source_file_path {
                            self.loaded_files.unload_file(&path);
                        }
                    }
                }
            }
        });
    }

    fn render_animations(&mut self, ui: &mut egui::Ui) {
        ui.heading("Animations");
        if ui.button("Add Animation").clicked() {
            self.input_data
                .animations
                .insert(*self.input_data_identifier_generator, ImputedAnimation::default());
            *self.input_data_identifier_generator += 1;
        }

        ui.separator();

        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            let mut removed_animations = Vec::new();
            for animation_index in 0..self.input_data.animations.len() {
                let (animation_identifier, animation) = &mut self.input_data.animations.get_index_mut(animation_index).unwrap();
                egui::CollapsingHeader::new(format!("Animation: {}", animation.name))
                    .id_salt(**animation_identifier)
                    .default_open(true)
                    .show(ui, |ui| {
                        let name_label = ui.label("Animation Name: ");
                        ui.text_edit_singleline(&mut animation.name).labelled_by(name_label.id);

                        ui.horizontal(|ui| {
                            if ui.button("Select Model File…").clicked() {
                                if let Some(path) = rfd::FileDialog::new()
                                    .set_title("Select Model File")
                                    .add_filter("Supported Files", &SUPPORTED_FILES)
                                    .pick_file()
                                {
                                    if let Some(last_path) = &animation.source_file_path {
                                        self.loaded_files.unload_file(last_path);
                                    };
                                    animation.source_file_path = Some(path.clone());
                                    self.loaded_files.load_file(path);
                                }
                            }

                            if ui.button("Remove Animation").clicked() {
                                removed_animations.push(**animation_identifier);
                            }
                        });

                        if let Some(source_file_path) = &animation.source_file_path {
                            let file_status = self.loaded_files.get_file_status(source_file_path).unwrap(); // FIXME: This must be check!

                            ui.horizontal(|ui| {
                                ui.label("Animation File:");
                                ui.monospace(source_file_path.display().to_string());
                                match file_status {
                                    FileStatus::Loading => {
                                        ui.spinner();
                                        animation.source_animation = 0;
                                    }
                                    FileStatus::Loaded(_) => {
                                        ui_success(ui);
                                    }
                                    FileStatus::Failed => {
                                        ui_failed(ui);
                                        animation.source_animation = 0;
                                    }
                                }
                            });

                            if let FileStatus::Loaded(file_data) = file_status {
                                if animation.source_animation > file_data.animations.len() {
                                    animation.source_animation = 0;
                                }

                                ui.separator();
                                egui::ComboBox::from_label("Source Animation")
                                    .selected_text(file_data.animations.get_index(animation.source_animation).unwrap().0)
                                    .show_ui(ui, |ui| {
                                        for (source_animation_index, (source_animation_name, _)) in file_data.animations.iter().enumerate() {
                                            ui.selectable_value(&mut animation.source_animation, source_animation_index, source_animation_name);
                                        }
                                    });
                            }
                        }
                    });
            }

            for removed_animation in removed_animations {
                let removed_animation = self.input_data.animations.shift_remove(&removed_animation);
                if let Some(animation) = removed_animation {
                    if let Some(path) = animation.source_file_path {
                        self.loaded_files.unload_file(&path);
                    }
                }
            }
        });
    }

    fn render_sequences(&mut self, ui: &mut egui::Ui) {
        ui.heading("Sequences");
        if ui.button("Add Sequence").clicked() {
            self.input_data
                .sequences
                .insert(*self.input_data_identifier_generator, ImputedSequence::default());
            *self.input_data_identifier_generator += 1;
        }

        ui.separator();

        egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
            let mut removed_sequences = Vec::new();

            for sequence_index in 0..self.input_data.sequences.len() {
                let (sequence_identifier, sequence) = &mut self.input_data.sequences.get_index_mut(sequence_index).unwrap();
                egui::CollapsingHeader::new(format!("Sequence: {}", sequence.name))
                    .id_salt(**sequence_identifier)
                    .default_open(true)
                    .show(ui, |ui| {
                        let name_label = ui.label("Sequence Name: ");
                        ui.text_edit_singleline(&mut sequence.name).labelled_by(name_label.id);

                        ui.horizontal(|ui| {
                            if ui.button("Remove Sequence").clicked() {
                                removed_sequences.push(**sequence_identifier);
                            }
                        });

                        ui.separator();

                        if self.input_data.animations.is_empty() {
                            ui.colored_label(egui::Color32::RED, "No Animations Created");

                            if !sequence.animations.is_empty() {
                                sequence.animations.clear();
                            }

                            return;
                        }

                        if sequence.animations.is_empty() {
                            sequence.animations = vec![vec![0]];
                        }

                        let sequence_animation = &mut sequence.animations[0][0];
                        egui::ComboBox::from_label("Selected Animation")
                            .selected_text(&self.input_data.animations.get_index(*sequence_animation).unwrap().1.name)
                            .show_ui(ui, |ui| {
                                for (animation_index, (_, animation)) in self.input_data.animations.iter().enumerate() {
                                    ui.selectable_value(sequence_animation, animation_index, &animation.name);
                                }
                            });
                    });
            }

            for removed_sequence in removed_sequences {
                self.input_data.sequences.shift_remove(&removed_sequence);
            }
        });
    }
}
