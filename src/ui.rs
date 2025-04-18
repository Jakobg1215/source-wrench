use eframe::egui;

pub fn toggle_ui_compact(ui: &mut egui::Ui, on: &mut bool) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y * egui::vec2(2.0, 1.0);
    let (rect, mut response) = ui.allocate_exact_size(desired_size, egui::Sense::click());
    if response.clicked() {
        *on = !*on;
        response.mark_changed();
    }
    response.widget_info(|| egui::WidgetInfo::selected(egui::WidgetType::Checkbox, ui.is_enabled(), *on, ""));

    if ui.is_rect_visible(rect) {
        let how_on = ui.ctx().animate_bool_responsive(response.id, *on);
        let visuals = ui.style().interact_selectable(&response, *on);
        let rect = rect.expand(visuals.expansion);
        let radius = 0.5 * rect.height();
        ui.painter().rect(rect, radius, visuals.bg_fill, visuals.bg_stroke, egui::StrokeKind::Inside);
        let circle_x = egui::lerp((rect.left() + radius)..=(rect.right() - radius), how_on);
        let center = egui::pos2(circle_x, rect.center().y);
        ui.painter().circle(center, 0.75 * radius, visuals.bg_fill, visuals.fg_stroke);
    }

    response
}

pub fn ui_failed(ui: &mut egui::Ui) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y;
    let (rect, response) = ui.allocate_exact_size(desired_size * egui::Vec2::default(), egui::Sense::empty());

    if ui.is_rect_visible(rect) {
        let stroke = egui::Stroke::new(2.0, ui.visuals().strong_text_color());
        let offset = egui::Vec2::new(desired_size * 0.2, desired_size * 0.2);
        let top_left = rect.left_top() + offset;
        let bottom_right = rect.right_bottom() - offset;
        let top_right = rect.right_top() + egui::Vec2::new(-offset.x, offset.y);
        let bottom_left = rect.left_bottom() + egui::Vec2::new(offset.x, -offset.y);

        ui.painter().line_segment([top_left, bottom_right], stroke);
        ui.painter().line_segment([top_right, bottom_left], stroke);
    }

    response
}

pub fn ui_success(ui: &mut egui::Ui) -> egui::Response {
    let desired_size = ui.spacing().interact_size.y - (ui.spacing().interact_size.y * 0.2);
    let (rect, response) = ui.allocate_exact_size(desired_size * egui::Vec2::default(), egui::Sense::empty());

    if ui.is_rect_visible(rect) {
        let stroke = egui::Stroke::new(2.0, ui.visuals().strong_text_color());
        let center = rect.center();
        let start = egui::Pos2::new(center.x - desired_size * 0.3, center.y);
        let mid = egui::Pos2::new(center.x - desired_size * 0.1, center.y + desired_size * 0.3);
        let end = egui::Pos2::new(center.x + desired_size * 0.4, center.y - desired_size * 0.3);

        ui.painter().line_segment([start, mid], stroke);
        ui.painter().line_segment([mid, end], stroke);
    }

    response
}
