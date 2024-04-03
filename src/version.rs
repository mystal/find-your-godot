pub fn get_full_version(version: &str) -> String {
    // TODO: Use a more thorough heuristic.
    if version.contains('-') {
        version.to_string()
    } else {
        format!("{}-stable", version)
    }
}
