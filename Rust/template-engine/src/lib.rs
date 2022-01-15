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


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
