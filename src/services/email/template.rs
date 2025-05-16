use std::collections::HashMap;

// Embed all templates and assets at compile time
const EMAIL_BASE_HTML: &str = include_str!("../../../templates/email/base.html");
const EMAIL_STYLE_CSS: &str = include_str!("../../../templates/email/styles.html");

// Email templates - HTML versions
const VERIFICATION_EMAIL_HTML: &str = include_str!("../../../templates/email/verification.html");
const PASSWORD_RESET_HTML: &str = include_str!("../../../templates/email/password_reset.html");

// Email templates - Text versions
const VERIFICATION_EMAIL_TEXT: &str =
    include_str!("../../../templates/email/verification_text.txt");
const PASSWORD_RESET_TEXT: &str = include_str!("../../../templates/email/password_reset_text.txt");

pub struct TemplateManager;

impl TemplateManager {
    // Render HTML email with parameters
    pub fn render_html(template_name: &str, params: HashMap<&str, &str>) -> String {
        let title = match template_name {
            "verification" => "Email Verification - Safatanc Connect",
            "password_reset" => "Password Reset - Safatanc Connect",
            _ => "Safatanc Connect",
        };

        // Get the content template based on the template name
        let content_template = match template_name {
            "verification" => VERIFICATION_EMAIL_HTML,
            "password_reset" => PASSWORD_RESET_HTML,
            _ => panic!("Unknown template: {}", template_name),
        };

        // Replace placeholders in the content template
        let content = Self::replace_placeholders(content_template, &params);

        // Create parameters for the base template
        let mut base_params = HashMap::new();
        base_params.insert("title", title);
        base_params.insert("styles", EMAIL_STYLE_CSS);
        base_params.insert("content", &content);

        // Render the base template with the content
        Self::replace_placeholders(EMAIL_BASE_HTML, &base_params)
    }

    // Render text email with parameters
    pub fn render_text(template_name: &str, params: HashMap<&str, &str>) -> String {
        // Get the text template based on the template name
        let text_template = match template_name {
            "verification" => VERIFICATION_EMAIL_TEXT,
            "password_reset" => PASSWORD_RESET_TEXT,
            _ => panic!("Unknown template: {}", template_name),
        };

        // Replace placeholders in the text template
        Self::replace_placeholders(text_template, &params)
    }

    // Replace placeholders in a template with actual values
    fn replace_placeholders(template: &str, params: &HashMap<&str, &str>) -> String {
        let mut result = template.to_string();

        for (key, value) in params {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }
}
