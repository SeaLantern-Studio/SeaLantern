pub(super) fn is_newer_version(remote: &str, local: &str) -> bool {
    let remote_parts = parse_version(remote);
    let local_parts = parse_version(local);

    for index in 0..remote_parts.len().max(local_parts.len()) {
        let remote_part = remote_parts.get(index).copied().unwrap_or(0);
        let local_part = local_parts.get(index).copied().unwrap_or(0);
        if remote_part > local_part {
            return true;
        }
        if remote_part < local_part {
            return false;
        }
    }

    false
}

fn parse_version(version: &str) -> Vec<u32> {
    version
        .split('.')
        .filter_map(|segment| segment.parse::<u32>().ok())
        .collect()
}
