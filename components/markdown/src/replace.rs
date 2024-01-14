pub use crate::context::RenderContext;

#[derive(Debug)]
struct Delimiter {
    pub left: String,
    pub right: String,
    pub left_to: String,
    pub right_to: String,
}
impl Delimiter {
    pub fn from(text: String) -> Self {
        let mut from_to = text.split("->");
        let mut from = from_to.next().unwrap().split("(.+)");
        let mut to = from_to.next().unwrap().split("(.+)");
        Self {
            left: from.next().unwrap().to_string(),
            right: from.next().unwrap().to_string(),
            left_to: to.next().unwrap().to_string(),
            right_to: to.next().unwrap().to_string(),
        }
    }
}

// not very efficiency
// replace some delimiter to shortcode like $(*)$
pub fn replace(rawcontent: &str, delimiters: &Vec<String>) -> String {
    let delimiters = delimiters.iter().map(|d| Delimiter::from(d.to_string())).collect::<Vec<_>>();

    let mut out: Vec<char> = Vec::new();
    let mut continuous_backquote_count = 0;
    let mut block_quoted = false;
    let mut inline_quoted = false;
    let chars: Vec<_> = rawcontent.chars().collect();

    let mut leftside_detacted: Option<usize> = None;
    let mut processing = 0;
    'text: for (i, c) in chars.iter().enumerate() {
        if processing != 0 {
            processing -= 1;
            continue;
        }
        if *c == '`' {
            continuous_backquote_count += 1;
        } else {
            continuous_backquote_count = 0;
        }
        if continuous_backquote_count == 3 {
            block_quoted = !block_quoted;
            continuous_backquote_count = 0;
        };
        if continuous_backquote_count == 1 {
            inline_quoted = !inline_quoted;
            continuous_backquote_count = 0;
        };
        if !block_quoted && !inline_quoted {
            match leftside_detacted {
                None => {
                    for (id, d) in delimiters.iter().enumerate() {
                        if chars.len() < i + d.left.len() + d.right.len() {
                            continue;
                        }
                        let left=String::from_iter(&chars[i..i + d.left.len()]) == d.left;
                        let right=String::from_iter(&chars[i + d.left.len()..i + d.left.len() + d.right.len()]) == d.right;
                        
                        if left && !right
                        {
                            out.extend(d.left_to.chars());
                            leftside_detacted = Some(id);
                            processing = d.left.len() - 1;
                            continue 'text;
                        }
                    }
                }
                Some(id) => {
                    let d=&delimiters[id];
                    if chars.len() >= i + d.right.len() && String::from_iter(&chars[i..i + d.right.len()]) == d.right {
                        out.extend(d.right_to.chars());
                        leftside_detacted = None;
                        processing = d.right.len() - 1;
                        continue 'text;
                    }
                }
            };
        }
        out.push(*c);
    }
    out.iter().collect()
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_inline() {
        let delimiters = vec!["$(.+)$->\\((.+)\\)".to_string(), "$$(.+)$$->\\[(.+)\\]".to_string()];
        assert_eq!(replace("$inline$", &delimiters), "\\(inline\\)");
        assert_eq!(replace("`$inline$`", &delimiters), "`$inline$`");
        assert_eq!(replace("```\n$inline$\n```", &delimiters), "```\n$inline$\n```");
    }

    #[test]
    fn test() {
        let delimiters = vec![
            "$(.+)$->{{ katex(body=\"(.+)\") }}".to_string(),
            "$$(.+)$$->{% katex(block=true) %}(.+){% end %}".to_string(),
        ];
        let o = replace(
            "
asdfas

$inline$

$$
block
$$

$$block$$

`$inline$`

```block
$$
block
$$
```
",
            &delimiters,
        );
        println!("{}", o)
    }
}
