//! ##  HTML template engine
//! 
//! ![template engine design](./pix/Design_of_the_template_engine.png)
//!

use std::collections::HashMap;

// Data structures

/// ContentType is the main data structure to classify the template string read 
/// from the template file. It is represented as enum and contains the list of 
/// possible token types read from the template file. 
/// As each statement (template string) is read from the template file, 
/// it is evaluated to check if it is one of the types defined in this enum.
///
// Each line in input can be of one of following types
#[derive(PartialEq, Debug)]
pub enum ContentType {
    Literal(String),
    TemplateVariable(ExpressionData),
    Tag(TagType),
    Unrecognized,
}

/// TagType is a supporting data structure that is used to indicate 
/// whether a template string corresponds to a for-tag (repetitive loop) 
/// or if-tag (display control)
#[derive(PartialEq, Debug)]
pub enum TagType {
    ForTag,
    IfTag,
}

/// A struct to store the result of the tokenization of the template string
#[derive(PartialEq, Debug)]
pub struct ExpressionData {
    pub head: Option<String>,
    pub variable: String,
    pub tail: Option<String>,
}

/// Checking if the two matching tags are contained within the input string
pub fn check_matching_pair(input: &str, symbol1: &str, symbol2: &str) -> bool {
    input.contains(symbol1) && input.contains(symbol2)
}

///  Checks if a symbol string, for example, '{%', is contained within another string.
pub fn check_symbol_string(input: &str, symbol: &str) -> bool {
    input.contains(symbol)
}

/// takes two parameters and returns the index 
/// where the second value is found within the first value. 
/// This makes it easy to split the template string into three parts 
/// – head, variable, and tail
pub fn get_index_for_symbol(input: &str, symbol: char) -> (bool, usize) {
    let mut characters = input.char_indices();
    let mut does_exist = false;
    let mut index :usize = 0;

    while let Some((i, c)) = characters.next() {
        if c == symbol {
            does_exist = true;
            index = i;
            break;
        }
    }

    (does_exist, index)
}

/// This parses an expression with a template variable,
/// parses it into head, variable, and tail components, and returns the results
pub fn get_expression_data(input_line: &str) -> ExpressionData {
    let (_h, i) = get_index_for_symbol(input_line, '{');
    let head = input_line[0..i].to_string();

    let (_j, k) = get_index_for_symbol(input_line, '}');
    let variable = input_line[i+1 + 1..k].to_string();

    let tail = input_line[k+1 + 1..].to_string();

    ExpressionData {
        head: Some(head),
        variable: variable,
        tail: Some(tail),
    }
}



/// Entry point for parser. Accepts an input statement 
/// and tokenizes it into one of an if tag, a for tag, or a template variable.
pub fn get_content_type(input_line: &str) -> ContentType {
    let is_tag_expression = check_matching_pair(&input_line, "{%", "%}");
    let is_for_tag = (  check_symbol_string(&input_line, "for") && 
                        check_symbol_string(&input_line, "in")
                     ) 
                     || check_symbol_string(&input_line, "endfor") ;
    let is_if_tag = check_symbol_string(&input_line, "if")
                  || check_symbol_string(&input_line, "endif");
    
    let is_template_variable = check_matching_pair(&input_line, "{{", "}}");
    
    let content_type;

    if is_tag_expression && is_for_tag { 
        content_type = ContentType::Tag(TagType::ForTag);
    } else if is_tag_expression && is_if_tag {
        content_type = ContentType::Tag(TagType::IfTag);
    } else if is_template_variable {
        let content = get_expression_data(&input_line);
        content_type = ContentType::TemplateVariable(content);
    } else if !is_tag_expression && !is_template_variable {
        content_type = ContentType::Literal(input_line.to_string());
    } else {
        content_type = ContentType::Unrecognized;
    }

    content_type
}

/// constructs the output html statement consisting of head, text content, and tail. 
/// To construct the text content, the template variables are replaced with 
/// the values from the context data
pub fn generate_html_template_var(content :ExpressionData, context :HashMap<String, String>) -> String {
    let mut html = String::new();

    if let Some(h) = content.head {
        html.push_str(&h);
    }

    if let Some(v) = context.get(&content.variable) {
        html.push_str(&v);
    }

    if let Some(t) = content.tail {
        html.push_str(&t);
    }

    html
}


// ----------------------------------------------------------------
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_index_for_symbol_test() {
        assert_eq!((true, 3), get_index_for_symbol("Hi {name} , welcome", '{'));
    }

    #[test]
    fn get_expression_data_test() {
        let expression_data = ExpressionData {
            head: Some("Hi ".to_string()),
            variable: "name".to_string(),
            tail: Some(" , welcome".to_string()),
        };

        assert_eq!(expression_data, get_expression_data("Hi {{name}} , welcome"));
    }

    #[test]
    fn check_symbol_string_test() {
        assert_eq!(true, check_symbol_string("{{Hello}}", "{{"));
    }

    #[test]
    fn check_matching_pair_test() {
        assert_eq!(true, check_matching_pair("{{Hello}}", "{{", "}}"));
    }

    #[test]
    fn check_template_var_test() {
        let content = ExpressionData{
            head: Some("Hi ".to_string()),
            variable: "name".to_string(),
            tail: Some(" , welcome".to_string()),
        };

        assert_eq!(ContentType::TemplateVariable(content),
                   get_content_type("Hi {{name}} , welcome")
                   );
    }
}
