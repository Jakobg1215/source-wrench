#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod import;
mod input;
mod process;
mod ui;
mod utilities;
mod write;

use eframe::egui::{self, Color32};
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
struct SourceWrenchApplication {
    tab_tree: DockState<SourceWrenchTabType>,
    input_data_identifier_generator: usize,
    input_data: ImputedCompilationData,
    loaded_files: FileManager,
}

impl Default for SourceWrenchApplication {
    fn default() -> Self {
        let mut tree = DockState::new(vec![SourceWrenchTabType::Main]);

        let [main_tab, _] = tree
            .main_surface_mut()
            .split_right(egui_dock::NodeIndex::root(), 0.5, vec![SourceWrenchTabType::Logging]);

        let [_, _] = tree.main_surface_mut().split_below(
            main_tab,
            0.3,
            vec![SourceWrenchTabType::BodyGroups, SourceWrenchTabType::Animations, SourceWrenchTabType::Sequences],
        );

        Self {
            tab_tree: tree,
            input_data_identifier_generator: Default::default(),
            input_data: Default::default(),
            loaded_files: Default::default(),
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
            .show_leaf_collapse_buttons(false)
            .show(
                ctx,
                &mut SourceWrenchTabManager {
                    new_tabs: &mut new_tabs,
                    input_data_identifier_generator: &mut self.input_data_identifier_generator,
                    input_data: &mut self.input_data,
                    loaded_files: &mut self.loaded_files,
                },
            );

        new_tabs.drain(..).for_each(|tab| {
            let existing_tab = self.tab_tree.iter_all_tabs().find(|(_, tab_type)| **tab_type == tab);

            if let Some(((existing_surface, existing_node), _)) = existing_tab {
                self.tab_tree.set_focused_node_and_surface((existing_surface, existing_node)); // FIXME: This does not select a tab in a window.
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

struct SourceWrenchTabManager<'a> {
    new_tabs: &'a mut Vec<SourceWrenchTabType>,
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
            SourceWrenchTabType::BodyGroups => String::from("Body Group").into(),
            SourceWrenchTabType::Animations => String::from("Animation").into(),
            SourceWrenchTabType::Sequences => String::from("Sequence").into(),
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

            let button_response = ui.add_enabled(!self.loaded_files.is_loading_files(), egui::Button::new("Compile Model"));
            if button_response.clicked() {
                // FIXME: Make this run on a different thread!

                if self.input_data.model_name.is_empty() {
                    log("Model name is empty!", LogLevel::Error);
                    return;
                }

                let mut model_name = self.input_data.model_name.clone();
                if !model_name.ends_with(".mdl") {
                    model_name.push_str(".mdl");
                }

                let processed_data = match process::process(self.input_data, self.loaded_files) {
                    Ok(data) => data,
                    Err(error) => {
                        log(format!("Fail To Compile Model: {}!", error), LogLevel::Error);
                        return;
                    }
                };

                log("Writing Files!", LogLevel::Info);

                match write::write_files(
                    self.input_data.model_name.clone(),
                    model_name,
                    processed_data,
                    export_path.to_string_lossy().to_string(),
                ) {
                    Ok(_) => {}
                    Err(error) => {
                        log(format!("Fail To Write Files: {}!", error), LogLevel::Error);
                        return;
                    }
                }

                log("Model compiled successfully!", LogLevel::Info);
            }
        }

        if ui.button("Body Groups").clicked() {
            self.new_tabs.push(SourceWrenchTabType::BodyGroups);
        }

        if ui.button("Animations").clicked() {
            self.new_tabs.push(SourceWrenchTabType::Animations);
        }

        if ui.button("Sequences").clicked() {
            self.new_tabs.push(SourceWrenchTabType::Sequences);
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
                    logging::LogLevel::Log => Color32::GRAY,
                    logging::LogLevel::Info => Color32::DARK_GREEN,
                    logging::LogLevel::Verbose => Color32::MAGENTA,
                    logging::LogLevel::Debug => Color32::CYAN,
                    logging::LogLevel::Warn => Color32::YELLOW,
                    logging::LogLevel::Error => Color32::RED,
                };

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
                                        if ui.button("Remove Model").clicked() {
                                            removed_models.push(**model_identifier);
                                        }

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
                                    });

                                    if let Some(source_file_path) = &model.source_file_path {
                                        let file_status = self.loaded_files.get_file_status(source_file_path).unwrap();

                                        ui.horizontal(|ui| {
                                            ui.label("Model File:");
                                            ui.monospace(source_file_path.display().to_string());
                                            match file_status {
                                                FileStatus::Loading => {
                                                    ui.spinner();
                                                    if model.enabled_source_parts.is_some() {
                                                        model.enabled_source_parts = None;
                                                    }
                                                }
                                                FileStatus::Loaded(_) => {
                                                    ui_success(ui);
                                                }
                                                FileStatus::Failed => {
                                                    ui_failed(ui);
                                                    if model.enabled_source_parts.is_some() {
                                                        model.enabled_source_parts = None;
                                                    }
                                                }
                                            }
                                        });

                                        if let FileStatus::Loaded(file_data) = file_status {
                                            if file_data.parts.is_empty() {
                                                ui.colored_label(Color32::RED, "Model File Has No Mesh!");
                                                return;
                                            }

                                            if model.enabled_source_parts.is_none() {
                                                model.enabled_source_parts = Some(vec![true; file_data.parts.len()]);
                                            }

                                            ui.heading("Enabled Parts");
                                            ui.separator();
                                            egui::ScrollArea::horizontal().show(ui, |ui| {
                                                for (part_index, (part_name, _)) in file_data.parts.iter().enumerate() {
                                                    let enabled_source_part = &mut model.enabled_source_parts.as_mut().unwrap()[part_index];
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
                            if ui.button("Remove Animation").clicked() {
                                removed_animations.push(**animation_identifier);
                            }

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
                        });

                        if let Some(source_file_path) = &animation.source_file_path {
                            let file_status = self.loaded_files.get_file_status(source_file_path).unwrap();

                            ui.horizontal(|ui| {
                                ui.label("Animation File:");
                                ui.monospace(source_file_path.display().to_string());
                                match file_status {
                                    FileStatus::Loading => {
                                        ui.spinner();
                                        if animation.source_animation.is_some() {
                                            animation.source_animation = Some(0);
                                        }
                                    }
                                    FileStatus::Loaded(_) => {
                                        ui_success(ui);
                                    }
                                    FileStatus::Failed => {
                                        ui_failed(ui);
                                        if animation.source_animation.is_some() {
                                            animation.source_animation = Some(0);
                                        }
                                    }
                                }
                            });

                            if let FileStatus::Loaded(file_data) = file_status {
                                if animation.source_animation.is_none() {
                                    animation.source_animation = Some(0);
                                }

                                ui.separator();
                                egui::ComboBox::from_label("Source Animation")
                                    .selected_text(file_data.animations.get_index(animation.source_animation.unwrap()).unwrap().0)
                                    .show_ui(ui, |ui| {
                                        for (source_animation_index, (source_animation_name, _)) in file_data.animations.iter().enumerate() {
                                            ui.selectable_value(&mut animation.source_animation, Some(source_animation_index), source_animation_name);
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
                            ui.colored_label(Color32::RED, "No Animations Created");

                            if sequence.animations.is_some() {
                                sequence.animations = None;
                            }

                            return;
                        }

                        if sequence.animations.is_none() {
                            sequence.animations = Some(vec![vec![0]]);
                        }

                        let sequence_animation = &mut sequence.animations.as_mut().unwrap()[0][0];
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
