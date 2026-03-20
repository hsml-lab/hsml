use nom::{
    IResult, Needed,
    bytes::complete::tag,
    error::{Error, ErrorKind},
};

use crate::parser::{Span, advance, take_prefix};

pub(super) fn process_class(input: Span<'_>) -> IResult<Span<'_>, Span<'_>> {
    let (input, _) = tag(".")(input)?;

    let mut remaining = input;

    let mut class_index = 0;

    loop {
        // get first char and check if it is a `[`
        // if so, it is an arbitrary tailwind value
        let first_char = remaining.fragment().chars().next();

        match first_char {
            Some('#') => {
                // we hit a id, so we are done
                break;
            }
            Some('.') => {
                // we hit a new class, so we are done
                break;
            }
            Some('(') => {
                // we hit the start of attributes, so we are done
                break;
            }
            Some(' ') => {
                // we hit a whitespace, so we are done
                break;
            }
            Some('\t') => {
                // we hit a tab, so we are done
                break;
            }
            Some('\r') if remaining.fragment().as_bytes().get(1) == Some(&b'\n') => {
                // we hit a newline, so we are done
                break;
            }
            Some('\r') => {}
            Some('\n') => {
                // we hit a newline, so we are done
                break;
            }
            Some('[') => {
                // Parse arbitrary tailwind values (https://tailwindcss.com/docs/adding-custom-styles#using-arbitrary-values)

                let closing_bracket = ']';

                let mut closing_bracket_index = 0;
                let mut is_escaped = false;

                for (index, c) in remaining.fragment().char_indices() {
                    if index == 0 {
                        // skip first char, because it is the opening bracket
                        continue;
                    }

                    if c == '\\' {
                        is_escaped = true;
                        continue;
                    }

                    if c == closing_bracket && !is_escaped {
                        closing_bracket_index = index;
                        break;
                    }

                    is_escaped = false;
                }

                if closing_bracket_index == 0 {
                    return Err(nom::Err::Error(Error::new(remaining, ErrorKind::Tag)));
                }

                class_index += closing_bracket_index;
                remaining = advance(input, class_index);

                continue;
            }
            Some(_) => {
                // we hit a char, so we need to append it to the class
                class_index += remaining.fragment().chars().next().unwrap().len_utf8();
                remaining = advance(input, class_index);
                continue;
            }
            None => {
                return Err(nom::Err::Incomplete(Needed::Unknown));
            }
        }
    }

    let class = take_prefix(input, class_index);

    Ok((remaining, class))
}
