use std::collections::HashMap;
use std::io;
use std::io::BufRead;
use template_engine::*;

/// The main() function performs the coordination role tying all pieces together.
/// It invokes the parser, initializes the context data, and then invokes the generator:
fn main() {
    // Pass context data:
    // It creates a HashMap to pass values for the template variables
    // mentioned in the template. We add values for name and city to this HashMap.
    // The HashMap is passed to the generator function along with the parsed template input
    let mut context: HashMap<String, String> = HashMap::new();
    context.insert("name".to_string(), "Bob".to_string());
    context.insert("city".to_string(), "Boston".to_string());

    // Invoke parser and generator:
    // The parser is invoked by the call to the get_context_data() function
    // for each line of input read from the command line (standard input).
    // ---
    // a) If the line contains template variable,
    // it invokes the HTML generator generate_html_template_var() to create the HTML output.
    // ---
    // b) If the line contains a literal string, it simply echoes back the input HTML literal string.
    // ---
    // c) If the line contains for or if tags, right now, we simply print out a statement
    // that the feature is not yet implemented.
    for line in io::stdin().lock().lines() {
        match get_content_type(&line.unwrap().clone()) {
            ContentType::TemplateVariable(content) => {
                let html = generate_html_template_var(content, context.clone());
                println!("{}", html);
            }
            ContentType::Literal(text) => println!("{}", text),
            ContentType::Tag(TagType::ForTag) => println!("For Tag not imnplemented"),
            ContentType::Tag(TagType::IfTag) => println!("If Tag not imnplemented"),
            ContentType::Unrecognized => println!("Unrecognized input"),
        }
    }
}
