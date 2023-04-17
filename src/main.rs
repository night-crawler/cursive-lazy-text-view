use cursive::Cursive;
use cursive::direction::Orientation;
use cursive::traits::{Nameable, Resizable, Scrollable};
use cursive::utils::markup::StyledString;
use cursive::view::ScrollStrategy;
use cursive::views::{Checkbox, EditView, LinearLayout, NamedView, Panel, ResizedView, TextView};

pub fn build_edit_view<S1, S2, F>(name: S1, initial: S2, on_edit: F) -> NamedView<EditView>
    where
        F: Fn(&mut Cursive, &str, usize) + 'static,
        S1: Into<String>,
        S2: Into<String>,
{
    EditView::new()
        .content(initial)
        .on_edit(on_edit)
        .with_name(name)
}

fn mk_tv() -> ResizedView<TextView> {
    let line = "Lorem Ipsum is simply dummy text of the printing and typesetting industry. Lorem Ipsum has been the industry's standard dummy text ever since the 1500s, when an unknown printer took a galley of type and scrambled it to make a type specimen book. It has survived not only five centuries, but also the leap into electronic typesetting, remaining essentially unchanged. It was popularised in the 1960s with the release of Letraset sheets containing Lorem Ipsum passages, and more recently with desktop publishing software like Aldus PageMaker including versions of Lorem Ipsum.";
    let mut tv = TextView::new("");

    for _ in 0..1000 {
        tv.append(StyledString::styled(line, cursive::theme::ColorStyle::primary()));
        tv.append(StyledString::styled(line, cursive::theme::ColorStyle::secondary()));
        tv.append(StyledString::styled(line, cursive::theme::ColorStyle::highlight_inactive()));
    }

    tv.full_screen()
}

fn main() {
    // Creates the cursive root - required for every application.
    let mut siv = cursive::default();

    let mut main_layout = LinearLayout::new(Orientation::Vertical);
    let mut filter_layout = LinearLayout::new(Orientation::Horizontal);


    let filter_edit_view =
        build_edit_view("filter", "", move |_, text_, _| {});

    let since_minutes_edit_view =
        build_edit_view("since_minutes", "", move |_, text_, _| {});

    let filter_tail_lines_edit_view =
        build_edit_view("since_minutes", "", move |_, text_, _| {});

    let cb_timestamps = Checkbox::new()
        .on_change(move |_, checked_| {})
        .checked()
        .with_name("cb1");

    let cb_previous = Checkbox::new()
        .on_change(move |_, checked_| {})
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
        .title("aaa")
        .with_name("aaa");

    siv.add_layer(panel);

    // Starts the event loop.
    siv.run();
}