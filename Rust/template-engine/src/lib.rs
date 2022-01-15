//! ##  HTML template engine
//! 
//! ![template engine design](./pix/Design_of_the_template_engine.png)
//!

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


/// Entry point for parser. Accepts an input statement 
/// and tokenizes it into one of an if tag, a for tag, or a template variable.
pub fn get_content_type(input_line: &str) -> ContentType {
    let is_tag_expression = check_matching_pair(&input_line, "{%", "%}");
    let is_for_tag = (  check_symbol_string(&input_line, "for") && 
                        check_symbol_string(&input_line, "in")
                     ) 
                     || check_symbol_string(&input_line, "endfor") ;
    //NOTE: tmp_sub
    ContentType::Unrecognized
}


#[cfg(test)]
mod tests {
    use super::*;

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
