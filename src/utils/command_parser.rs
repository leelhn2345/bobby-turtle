use teloxide::utils::command::ParseError;

pub fn message_to_send(input: String) -> Result<(i64, String), ParseError> {
    let mut parts = input.splitn(2, ' ');

    let chat_id = parts
        .next()
        .unwrap_or_default()
        .parse::<i64>()
        .map_err(|e| ParseError::IncorrectFormat(e.into()))?;

    let message = parts.next().unwrap_or("yo yo yo").into();

    Ok((chat_id, message))
}
