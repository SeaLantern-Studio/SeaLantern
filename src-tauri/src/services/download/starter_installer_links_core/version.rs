use std::cmp::Ordering;
use std::collections::HashMap;

pub(super) fn choose_more_specific_bucket<'a>(
    selected: &mut Option<(&'a String, &'a HashMap<String, String>)>,
    version: &'a String,
    files: &'a HashMap<String, String>,
) {
    let should_replace = match selected {
        None => true,
        Some((selected_version, selected_files)) => {
            files.len() > selected_files.len()
                || (files.len() == selected_files.len()
                    && compare_version_keys_numeric(version, selected_version).is_gt())
        }
    };

    if should_replace {
        *selected = Some((version, files));
    }
}

pub(super) fn compare_version_keys_numeric(left: &str, right: &str) -> Ordering {
    let mut left_index = 0usize;
    let mut right_index = 0usize;

    loop {
        let left_token = next_version_token(left, &mut left_index);
        let right_token = next_version_token(right, &mut right_index);

        let ordering = match (left_token, right_token) {
            (None, None) => return left.cmp(right),
            (Some(_), None) => Ordering::Greater,
            (None, Some(_)) => Ordering::Less,
            (Some(VersionToken::Numeric(left_num)), Some(VersionToken::Numeric(right_num))) => {
                compare_numeric_token(left_num, right_num)
            }
            (Some(VersionToken::Text(left_text)), Some(VersionToken::Text(right_text))) => {
                compare_text_token(left_text, right_text)
            }
            (Some(VersionToken::Numeric(_)), Some(VersionToken::Text(_))) => Ordering::Greater,
            (Some(VersionToken::Text(_)), Some(VersionToken::Numeric(_))) => Ordering::Less,
        };

        if ordering != Ordering::Equal {
            return ordering;
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum VersionToken<'a> {
    Numeric(&'a str),
    Text(&'a str),
}

fn next_version_token<'a>(value: &'a str, index: &mut usize) -> Option<VersionToken<'a>> {
    let bytes = value.as_bytes();

    while *index < bytes.len() && !bytes[*index].is_ascii_alphanumeric() {
        *index += 1;
    }

    if *index >= bytes.len() {
        return None;
    }

    let start = *index;
    if bytes[*index].is_ascii_digit() {
        while *index < bytes.len() && bytes[*index].is_ascii_digit() {
            *index += 1;
        }
        return Some(VersionToken::Numeric(&value[start..*index]));
    }

    while *index < bytes.len() && bytes[*index].is_ascii_alphabetic() {
        *index += 1;
    }
    Some(VersionToken::Text(&value[start..*index]))
}

fn compare_numeric_token(left: &str, right: &str) -> Ordering {
    let left_trimmed = left.trim_start_matches('0');
    let right_trimmed = right.trim_start_matches('0');
    let left_normalized = if left_trimmed.is_empty() { "0" } else { left_trimmed };
    let right_normalized = if right_trimmed.is_empty() { "0" } else { right_trimmed };

    left_normalized
        .len()
        .cmp(&right_normalized.len())
        .then_with(|| left_normalized.cmp(right_normalized))
}

fn compare_text_token(left: &str, right: &str) -> Ordering {
    let case_insensitive = left.to_ascii_lowercase().cmp(&right.to_ascii_lowercase());
    if case_insensitive != Ordering::Equal {
        return case_insensitive;
    }
    left.cmp(right)
}
