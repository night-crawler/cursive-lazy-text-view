use cursive::direction::Orientation;
use cursive::theme::ColorStyle;
use cursive::traits::{Nameable, Resizable, Scrollable};
use cursive::utils::markup::StyledString;
use cursive::view::ScrollStrategy;
use cursive::views::{Checkbox, EditView, LinearLayout, Panel, ResizedView};

use crate::ltv::TextView;
// use cursive::views::TextView;

mod ltv;


fn mk_tv() -> ResizedView<TextView> {
    let line = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.";
    let mut tv = TextView::new("");

    for _ in 0..100 {
        for color in [ColorStyle::primary(), ColorStyle::secondary(), ColorStyle::highlight_inactive()].iter() {
            tv.append(StyledString::styled(line, *color));
        }
    }

    tv.full_screen()
}

fn main() {
    let mut siv = cursive::default();

    let mut main_layout = LinearLayout::new(Orientation::Vertical);
    let mut filter_layout = LinearLayout::new(Orientation::Horizontal);

    let filter_edit_view = EditView::new();
    let since_minutes_edit_view = EditView::new();
    let filter_tail_lines_edit_view = EditView::new();

    let cb_timestamps = Checkbox::new()
        .checked()
        .with_name("cb1");

    let cb_previous = Checkbox::new()
        .checked()
        .with_name("prev");

    let filter_edit_view_panel = Panel::new(filter_edit_view).title("Search").full_width();
    let since_minutes_panel = Panel::new(since_minutes_edit_view).title("Since minutes");
    let filter_tail_lines_panel = Panel::new(filter_tail_lines_edit_view).title("Tail lines");
    let cb_timestamps_panel = Panel::new(cb_timestamps).title("Timestamps");
    let cb_previous_panel = Panel::new(cb_previous).title("Previous");

    filter_layout.add_child(filter_edit_view_panel);
    filter_layout.add_child(cb_timestamps_panel);
    filter_layout.add_child(cb_previous_panel);
    filter_layout.add_child(since_minutes_panel);
    filter_layout.add_child(filter_tail_lines_panel);

    main_layout.add_child(filter_layout.full_width());

    let tv = mk_tv().scrollable()
        .scroll_x(true)
        .scroll_y(true)
        .scroll_strategy(ScrollStrategy::StickToBottom);
    main_layout.add_child(tv);

    let panel = Panel::new(main_layout)
        .title("Sample")
        .with_name("Sample");

    siv.add_layer(panel);
    siv.run();
}
