//! Link interception for cmd:// URLs

/// Edit commands that can be triggered by clicking links
#[derive(Debug, Clone, PartialEq)]
pub enum EditCommand {
    /// Edit the title
    Title,
    /// Edit the subtitle
    Subtitle,
    /// Edit the body
    Body,
    /// Edit the image URL
    Image,
    /// Edit a metadata field
    Metadata(String),
}

/// Parse a cmd:// URL into an EditCommand
///
/// Returns None if the URL is not a valid cmd:// URL
pub fn parse_cmd_url(url: &str) -> Option<EditCommand> {
    // Check if it's a cmd:// URL
    let path = url.strip_prefix("cmd://")?;

    // Parse the command
    let parts: Vec<&str> = path.split('/').collect();

    match parts.as_slice() {
        ["edit", "title"] => Some(EditCommand::Title),
        ["edit", "subtitle"] => Some(EditCommand::Subtitle),
        ["edit", "body"] => Some(EditCommand::Body),
        ["edit", "image"] => Some(EditCommand::Image),
        ["edit", "meta", key] => Some(EditCommand::Metadata((*key).to_string())),
        _ => None,
    }
}
