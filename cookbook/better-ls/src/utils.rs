use std::time::SystemTime;

/// Format file size in human readable format
pub fn format_size(size: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = size as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format file permissions
pub fn format_permissions(metadata: &std::fs::Metadata) -> String {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = metadata.permissions().mode();
        let user = if mode & 0o400 != 0 { "r" } else { "-" };
        let user = format!("{}{}", user, if mode & 0o200 != 0 { "w" } else { "-" });
        let user = format!("{}{}", user, if mode & 0o100 != 0 { "x" } else { "-" });

        let group = if mode & 0o040 != 0 { "r" } else { "-" };
        let group = format!("{}{}", group, if mode & 0o020 != 0 { "w" } else { "-" });
        let group = format!("{}{}", group, if mode & 0o010 != 0 { "x" } else { "-" });

        let other = if mode & 0o004 != 0 { "r" } else { "-" };
        let other = format!("{}{}", other, if mode & 0o002 != 0 { "w" } else { "-" });
        let other = format!("{}{}", other, if mode & 0o001 != 0 { "x" } else { "-" });

        format!("{}{}{}", user, group, other)
    }
    #[cfg(not(unix))]
    {
        if metadata.permissions().readonly() {
            "r--r--r--".to_string()
        } else {
            "rw-rw-rw-".to_string()
        }
    }
}

/// Format system time to human readable string
pub fn format_time(time: SystemTime) -> String {
    use std::time::UNIX_EPOCH;

    match time.duration_since(UNIX_EPOCH) {
        Ok(duration) => {
            let secs = duration.as_secs();
            let datetime = chrono::DateTime::from_timestamp(secs as i64, 0)
                .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap());
            datetime.format("%Y-%m-%d %H:%M").to_string()
        }
        Err(_) => "Unknown".to_string(),
    }
}
