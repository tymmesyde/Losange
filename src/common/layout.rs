use gtk::prelude::AccessibleExt;
use gtk::prelude::AdjustmentExt;
use gtk::prelude::Cast;
use gtk::prelude::ObjectExt;
use gtk::prelude::StaticType;
use gtk::prelude::WidgetExt;
use gtk::Orientation;
use gtk::ScrolledWindow;
use itertools::Itertools;
use relm4::prelude::*;
use relm4::ContainerChild;
use relm4::RelmIterChildrenExt;

pub fn in_view<C>(
    items: &FactoryVecDeque<C>,
    scrolled_window: &ScrolledWindow,
    orientation: Orientation,
) -> Vec<usize>
where
    C: FactoryComponent<Index = DynamicIndex>,
    C::ParentWidget: RelmIterChildrenExt,
    <C::ParentWidget as ContainerChild>::Child: AccessibleExt,
{
    let adjustment = match orientation {
        Orientation::Vertical => scrolled_window.vadjustment(),
        _ => scrolled_window.hadjustment(),
    };

    let min = adjustment.value();
    let max = adjustment.value() + adjustment.page_size();

    items
        .widget()
        .iter_children()
        .map(|child| match child.bounds() {
            Some((x, y, width, height)) => match orientation {
                Orientation::Vertical => (y, y + height),
                _ => (x, x + width),
            },
            None => (0, 0),
        })
        .enumerate()
        // .filter(|(.., (start, end))| max > min && end > start && start >= &0)
        .filter(|(.., (start, end))| *start as f64 <= max && *end as f64 >= min)
        .map(|(index, ..)| index)
        .collect_vec()
}

pub fn nested_scrolled_window() -> ScrolledWindow {
    let scrolled_window = ScrolledWindow::new();
    let controllers = scrolled_window.observe_controllers();

    controllers.into_iter().for_each(|collection| {
        if let Ok(event) = collection {
            if event.type_() == gtk::EventControllerScroll::static_type() {
                let scroll_controller = event
                    .downcast::<gtk::EventControllerScroll>()
                    .expect("Failed to downcast event controller to gtk::EventControllerScroll>");
                scroll_controller.set_flags(gtk::EventControllerScrollFlags::HORIZONTAL);
            }
        }
    });

    scrolled_window
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScrollPosition {
    Start,
    End,
    Middle,
    None,
}

pub fn horizontal_scroll_position(scrolled_window: &ScrolledWindow) -> ScrollPosition {
    let adjustment = scrolled_window.hadjustment();
    let value = adjustment.value();
    let upper = adjustment.upper();
    let page_size = adjustment.page_size();

    let viewport = page_size + value;
    match (value == 0.0, viewport == upper) {
        (true, false) => ScrollPosition::Start,
        (false, true) => ScrollPosition::End,
        (false, false) => ScrollPosition::Middle,
        _ => ScrollPosition::None,
    }
}

pub fn scrolled_to_bottom(scrolled_window: &ScrolledWindow) -> bool {
    let adjustment = scrolled_window.vadjustment();
    let value = adjustment.value();
    let upper = adjustment.upper();
    let page_size = adjustment.page_size();

    value + page_size == upper
}
