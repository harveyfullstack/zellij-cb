use unicode_width::UnicodeWidthStr;

use crate::LinePart;
use crate::UserConfiguration;
use zellij_tile::prelude::*;
use zellij_tile_utils::style;

fn get_current_title_len(current_title: &[LinePart]) -> usize {
    current_title.iter().map(|p| p.len).sum()
}

// move elements from before_active and after_active into tabs_to_render while they fit in cols
// adds collapsed_tabs to the left and right if there's left over tabs that don't fit
fn populate_tabs_in_tab_line(
    tabs_before_active: &mut Vec<LinePart>,
    tabs_after_active: &mut Vec<LinePart>,
    tabs_to_render: &mut Vec<LinePart>,
    cols: usize,
    user_conf: UserConfiguration,
) {
    let mut middle_size = get_current_title_len(tabs_to_render);

    let mut total_left = 0;
    let mut total_right = 0;
    loop {
        let left_count = tabs_before_active.len();
        let right_count = tabs_after_active.len();

        // left_more_tab_index is the tab to the left of the leftmost visible tab
        let left_more_tab_index = left_count.saturating_sub(1);
        let collapsed_left = left_more_message(left_count, user_conf.clone(), left_more_tab_index);
        // right_more_tab_index is the tab to the right of the rightmost visible tab
        let right_more_tab_index = left_count + tabs_to_render.len();
        let collapsed_right =
            right_more_message(right_count, user_conf.clone(), right_more_tab_index);

        let total_size = collapsed_left.len + middle_size + collapsed_right.len;

        if total_size > cols {
            // break and dont add collapsed tabs to tabs_to_render, they will not fit
            break;
        }

        let left = if let Some(tab) = tabs_before_active.last() {
            tab.len
        } else {
            usize::MAX
        };

        let right = if let Some(tab) = tabs_after_active.first() {
            tab.len
        } else {
            usize::MAX
        };

        // total size is shortened if the next tab to be added is the last one, as that will remove the collapsed tab
        let size_by_adding_left =
            left.saturating_add(total_size)
                .saturating_sub(if left_count == 1 {
                    collapsed_left.len
                } else {
                    0
                });
        let size_by_adding_right =
            right
                .saturating_add(total_size)
                .saturating_sub(if right_count == 1 {
                    collapsed_right.len
                } else {
                    0
                });

        let left_fits = size_by_adding_left <= cols;
        let right_fits = size_by_adding_right <= cols;
        // active tab is kept in the middle by adding to the side that
        // has less width, or if the tab on the other side doesn't fit
        if (total_left <= total_right || !right_fits) && left_fits {
            // add left tab
            let tab = tabs_before_active.pop().unwrap();
            middle_size += tab.len;
            total_left += tab.len;
            tabs_to_render.insert(0, tab);
        } else if right_fits {
            // add right tab
            let tab = tabs_after_active.remove(0);
            middle_size += tab.len;
            total_right += tab.len;
            tabs_to_render.push(tab);
        } else {
            // there's either no space to add more tabs or no more tabs to add, so we're done
            tabs_to_render.insert(0, collapsed_left);
            tabs_to_render.push(collapsed_right);
            break;
        }
    }
}

fn left_more_message(
    tab_count_to_the_left: usize,
    user_conf: UserConfiguration,
    tab_index: usize,
) -> LinePart {
    if tab_count_to_the_left == 0 {
        return LinePart::default();
    }
    let more_text = if tab_count_to_the_left < 10000 {
        format!(" ← +{} ", tab_count_to_the_left)
    } else {
        " ← +many ".to_string()
    };
    let more_text_len = more_text.len();
    let more_styled_text = style!(user_conf.color_fg, user_conf.color_others)
        .bold()
        .paint(more_text);
    LinePart {
        part: more_styled_text.to_string(),
        len: more_text_len,
        tab_index: Some(tab_index),
    }
}

fn right_more_message(
    tab_count_to_the_right: usize,
    user_conf: UserConfiguration,
    tab_index: usize,
) -> LinePart {
    if tab_count_to_the_right == 0 {
        return LinePart::default();
    };
    let more_text = if tab_count_to_the_right < 10000 {
        format!(" +{} → ", tab_count_to_the_right)
    } else {
        " +many → ".to_string()
    };
    let more_text_len = more_text.len();
    let more_styled_text = style!(user_conf.color_fg, user_conf.color_others)
        .bold()
        .paint(more_text);
    LinePart {
        part: more_styled_text.to_string(),
        len: more_text_len,
        tab_index: Some(tab_index),
    }
}

fn tab_line_prefix(
    session_name: String,
    mode: InputMode,
    user_conf: UserConfiguration,
    cols: usize,
    session_directory: String,
) -> Vec<LinePart> {
    let mut parts: Vec<LinePart> = Vec::new();

    let bg_color = user_conf.color_bg;
    let normal_mode_color = user_conf.color_normal_mode;
    let other_modes_color = user_conf.color_other_modes;

    let session_name_separator = "-";
    let session_name_parts = session_name
        .split(session_name_separator)
        .collect::<Vec<_>>();
    let session_name_parts_len = session_name_parts.len();
    let mut prefix_text_len = 0;

    let has_prefix = user_conf.display_session_directory || session_name_parts_len > 2;

    if has_prefix {
        let prefix_text = if user_conf.display_session_directory {
            session_directory
        } else {
            session_name_parts[..session_name_parts_len - 2].join(session_name_separator)
        };
        prefix_text_len = prefix_text.chars().count();
        let text_color = user_conf.color_session_directory;

        let prefix_styled_text = style!(text_color, bg_color).bold().paint(prefix_text);
        parts.push(LinePart {
            part: prefix_styled_text.to_string(),
            len: prefix_text_len,
            tab_index: None,
        });
    }

    let name_part = format!(
        "{}{} ",
        if has_prefix {
            session_name_separator
        } else {
            ""
        },
        if user_conf.display_session_directory {
            session_name
        } else {
            if session_name_parts_len == 1 {
                session_name
            } else {
                session_name_parts[session_name_parts_len - 2..session_name_parts_len]
                    .join(session_name_separator)
            }
        }
    );
    let name_part_len = name_part.width();
    let text_color = user_conf.color_session_name;
    let name_part_styled_text = style!(text_color, bg_color)
        .bold()
        .italic()
        .paint(name_part);
    if cols.saturating_sub(prefix_text_len) >= name_part_len {
        parts.push(LinePart {
            part: name_part_styled_text.to_string(),
            len: name_part_len,
            tab_index: None,
        })
    }

    let mut mode_part = user_conf.mode_display.get(&mode).unwrap().to_owned();
    mode_part.push(' ');
    let mode_part_len = mode_part.width();
    let mode_part_styled_text = match mode {
        InputMode::Normal => style!(normal_mode_color, bg_color).bold().paint(mode_part),
        _ => style!(other_modes_color, bg_color).bold().paint(mode_part),
    };
    if cols.saturating_sub(prefix_text_len) >= mode_part_len {
        parts.push(LinePart {
            part: mode_part_styled_text.to_string(),
            len: mode_part_len,
            tab_index: None,
        })
    }
    parts
}

pub fn tab_line(
    session_name: String,
    mut all_tabs: Vec<LinePart>,
    active_tab_index: usize,
    cols: usize,
    user_conf: UserConfiguration,
    mode: InputMode,
    session_directory: String,
) -> Vec<LinePart> {
    let mut tabs_after_active = all_tabs.split_off(active_tab_index);
    let mut tabs_before_active = all_tabs;
    let active_tab = if !tabs_after_active.is_empty() {
        tabs_after_active.remove(0)
    } else {
        tabs_before_active.pop().unwrap()
    };
    let mut prefix = tab_line_prefix(
        session_name,
        mode,
        user_conf.clone(),
        cols,
        session_directory,
    );
    let prefix_len = get_current_title_len(&prefix);

    // if active tab alone won't fit in cols, don't draw any tabs
    if prefix_len + active_tab.len > cols {
        return prefix;
    }

    let mut tabs_to_render = vec![active_tab];

    populate_tabs_in_tab_line(
        &mut tabs_before_active,
        &mut tabs_after_active,
        &mut tabs_to_render,
        cols.saturating_sub(prefix_len),
        user_conf,
    );
    prefix.append(&mut tabs_to_render);

    prefix
}
