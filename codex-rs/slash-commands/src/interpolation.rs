use crate::errors::SlashCommandError;
use crate::models::context::InterpolationContext;

const ARGUMENTS_KEY: &str = "ARGUMENTS";

pub fn interpolate_template(
    template: &str,
    ctx: &InterpolationContext,
) -> Result<String, SlashCommandError> {
    let mut output = String::with_capacity(template.len());
    let mut chars = template.char_indices().peekable();
    let mut cached_all_args: Option<String> = None;

    while let Some((idx, ch)) = chars.next() {
        if ch != '$' {
            output.push(ch);
            continue;
        }

        let after_dollar = idx + ch.len_utf8();
        let remaining = &template[after_dollar..];
        if remaining.starts_with(ARGUMENTS_KEY) {
            let all_args = cached_all_args.get_or_insert_with(|| ctx.all_arguments().to_string());
            output.push_str(all_args);
            for _ in 0..ARGUMENTS_KEY.len() {
                let _ = chars.next();
            }
            continue;
        }

        let mut digit_count = 0;
        for c in remaining.chars() {
            if c.is_ascii_digit() {
                digit_count += 1;
            } else {
                break;
            }
        }

        if digit_count == 0 {
            output.push('$');
            continue;
        }

        let end = after_dollar + digit_count;
        let number_slice = &template[after_dollar..end];
        let index = number_slice.parse::<usize>().map_err(|err| {
            SlashCommandError::Interpolation(format!(
                "invalid positional index '{number_slice}': {err}"
            ))
        })?;
        let replacement = ctx.positional(index).unwrap_or("");
        output.push_str(replacement);
        for _ in 0..digit_count {
            let _ = chars.next();
        }
    }

    Ok(output)
}
