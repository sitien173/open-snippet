const CURSOR_TOKEN: &str = "$|$";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CursorTokenError {
    TooManyTokens,
}

pub fn strip_cursor_token(input: &str) -> Result<(String, Option<usize>), CursorTokenError> {
    let mut matches = input.match_indices(CURSOR_TOKEN);
    let Some((index, _)) = matches.next() else {
        return Ok((input.to_string(), None));
    };

    if matches.next().is_some() {
        return Err(CursorTokenError::TooManyTokens);
    }

    let before = &input[..index];
    let after = &input[index + CURSOR_TOKEN.len()..];
    let mut text = String::with_capacity(input.len() - CURSOR_TOKEN.len());
    text.push_str(before);
    text.push_str(after);

    Ok((text, Some(after.encode_utf16().count())))
}
