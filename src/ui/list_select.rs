use eframe::egui::{Align, Context, Frame, Id, Layout, Response, ScrollArea, Sense, TextEdit, Ui, UiBuilder, Widget, vec2};

use crate::ui::{IconType, icon_button};

#[derive(Debug)]
pub struct ListSelect {
    id_salt: Id,
}

#[derive(Clone, Debug, Default)]
struct ListSelectState {
    selected_index: usize,
}

impl ListSelectState {
    pub fn load(ctx: &Context, id: Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_persisted(id))
    }

    pub fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_persisted(id, self));
    }
}

impl ListSelect {
    pub fn new(id_salt: impl std::hash::Hash) -> Self {
        Self { id_salt: Id::new(id_salt) }
    }

    pub fn show<'a, T: Default>(self, ui: &mut Ui, items: &'a mut Vec<T>, get_name_field: fn(&mut T) -> &mut String) -> Option<&'a mut T> {
        let ctx = ui.ctx().clone();
        let mut state = ListSelectState::load(&ctx, self.id_salt).unwrap_or_default();

        ui.with_layout(Layout::right_to_left(Align::LEFT), |ui| {
            ui.with_layout(Layout::top_down(Align::BOTTOM), |ui| {
                if ui.add(icon_button(IconType::Add)).clicked() {
                    state.selected_index = items.len();
                    items.push(T::default());
                }

                if ui.add(icon_button(IconType::Remove)).clicked() && !items.is_empty() {
                    items.remove(state.selected_index);
                    state.selected_index = state.selected_index.saturating_sub(1);
                }

                ui.add_space(ui.spacing().icon_width);

                if ui.add(icon_button(IconType::Up)).clicked() && state.selected_index > 0 {
                    items.swap(state.selected_index, state.selected_index - 1);
                    state.selected_index = state.selected_index.saturating_sub(1);
                }

                if ui.add(icon_button(IconType::Down)).clicked() && state.selected_index < items.len() - 1 {
                    items.swap(state.selected_index, state.selected_index + 1);
                    state.selected_index = state.selected_index.saturating_add(1);
                }
            });

            // Filter items for names.

            Frame::default()
                .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                .inner_margin(6)
                .corner_radius(4)
                .fill(ui.style().visuals.extreme_bg_color)
                .show(ui, |ui| {
                    ui.with_layout(Layout::top_down(Align::TOP), |ui| {
                        let row_height = ui.spacing().interact_size.y;
                        // TODO: Add draggable button to change the hight of the list.
                        ScrollArea::vertical().auto_shrink([false, false]).max_height(row_height * 6.0).show_rows(
                            ui,
                            row_height,
                            items.len(),
                            |ui, row_range| {
                                for row in row_range {
                                    let name = get_name_field(&mut items[row]);
                                    if list_item_entry(ui, name, row == state.selected_index).clicked() {
                                        state.selected_index = row;
                                    }
                                }
                            },
                        );

                        // TODO: Add Filtering Input
                        // CollapsingHeader::new("").id_salt(self.id_salt).show_unindented(ui, |ui| {
                        //     ui.label("Test");
                        // })
                    });
                });
        });

        let selected_index = state.selected_index;
        state.store(&ctx, self.id_salt);

        if items.is_empty() {
            return None;
        }

        Some(&mut items[selected_index])
    }
}

// TODO: Make it so you have to double click to edit it.
fn list_item_entry(ui: &mut Ui, name: &mut String, active: bool) -> Response {
    let width = ui.available_width();
    let height = ui.spacing().interact_size.y;
    let (rect, response) = ui.allocate_at_least(vec2(width, height), Sense::click());
    let visuals = ui.style().interact_selectable(&response, active);
    let mut child_ui = ui.new_child(UiBuilder::new().max_rect(rect));
    TextEdit::singleline(name)
        .background_color(visuals.bg_fill)
        .desired_width(width)
        .ui(&mut child_ui)
}
