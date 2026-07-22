/// Represents the type of bracket enclosing a token.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Bracket {
    None,
    Square,
    Parentheses,
    Curly,
}

/// Category tag for identified filename tokens.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Category {
    Year,
    Resolution,
    Source,
    VideoCodec,
    VideoProfile,
    ColorDepth,
    AudioCodec,
    AudioChannels,
    Season,
    Episode,
    Language,
    Container,
    Website,
    Other,
}

/// Represents a single token extracted from a filename.
#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub text: &'a str,
    pub start: usize,
    pub end: usize,
    pub bracket: Bracket,
    pub category: Option<Category>,
}

impl<'a> Token<'a> {
    /// Returns true if this token has been assigned a metadata category.
    #[inline]
    pub fn is_tagged(&self) -> bool {
        self.category.is_some()
    }
}

/// Splits the filename into a list of tokens based on delimiters and bracket transitions.
pub fn tokenize(filename: &str) -> Vec<Token<'_>> {
    // Pre-scan for valid matched brackets as pairs of (open_idx, close_idx)
    let mut valid_brackets: Vec<(usize, usize)> = Vec::with_capacity(8);
    let mut stack: Vec<(char, usize)> = Vec::with_capacity(8);
    
    for (i, c) in filename.char_indices() {
        match c {
            '[' | '(' | '{' => {
                stack.push((c, i));
            }
            ']' | ')' | '}' => {
                let expected_open = match c {
                    ']' => '[',
                    ')' => '(',
                    '}' => '{',
                    _ => unreachable!(),
                };
                if let Some(pos) = stack.iter().rposition(|&(open_c, _)| open_c == expected_open) {
                    let (_, open_idx) = stack.remove(pos);
                    valid_brackets.push((open_idx, i));
                }
            }
            _ => {}
        }
    }

    let mut tokens = Vec::with_capacity(16);
    let mut current_bracket = Bracket::None;
    let mut bracket_stack: Vec<(Bracket, usize)> = Vec::with_capacity(8);
    let mut token_start: Option<usize> = None;

    let find_close_idx = |open_idx: usize| -> Option<usize> {
        valid_brackets.iter().find(|&&(o, _)| o == open_idx).map(|&(_, c)| c)
    };

    for (i, c) in filename.char_indices() {
        match c {
            '[' | '(' | '{' if find_close_idx(i).is_some() => {
                if let Some(start) = token_start
                    && i > start {
                        tokens.push(Token {
                            text: &filename[start..i],
                            start,
                            end: i,
                            bracket: current_bracket,
                            category: None,
                        });
                    }
                let next_bracket = match c {
                    '[' => Bracket::Square,
                    '(' => Bracket::Parentheses,
                    '{' => Bracket::Curly,
                    _ => Bracket::None,
                };
                let close_idx = find_close_idx(i).unwrap();
                bracket_stack.push((next_bracket, close_idx));
                current_bracket = next_bracket;
                token_start = None;
            }
            ']' | ')' | '}' if bracket_stack.last().map(|&(_, close_idx)| close_idx) == Some(i) => {
                if let Some(start) = token_start
                    && i > start {
                        tokens.push(Token {
                            text: &filename[start..i],
                            start,
                            end: i,
                            bracket: current_bracket,
                            category: None,
                        });
                    }
                bracket_stack.pop();
                current_bracket = bracket_stack.last().map(|&(b, _)| b).unwrap_or(Bracket::None);
                token_start = None;
            }
            '.' | '_' | '-' | ' ' | '+' | ',' | '/' | '\\' | ':' | '*' | '×' => {
                if let Some(start) = token_start
                    && i > start {
                        tokens.push(Token {
                            text: &filename[start..i],
                            start,
                            end: i,
                            bracket: current_bracket,
                            category: None,
                        });
                    }
                token_start = None;
            }
            _ => {
                if token_start.is_none() {
                    token_start = Some(i);
                }
            }
        }
    }

    if let Some(start) = token_start {
        let len = filename.len();
        if len > start {
            tokens.push(Token {
                text: &filename[start..len],
                start,
                end: len,
                bracket: current_bracket,
                category: None,
            });
        }
    }

    tokens
}
