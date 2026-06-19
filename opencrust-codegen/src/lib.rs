//! OpenCrust Codegen public API.

use once_cell::sync::Lazy;
use std::sync::Arc;

mod adapters;
mod engine;
mod errors;
mod output;

pub use engine::Engine;
pub use errors::{CodegenError, Result};
pub use output::write_to;

/// Global engine instance – lazily initialised.
static GLOBAL_ENGINE: Lazy<Arc<Engine>> =
    Lazy::new(|| Arc::new(Engine::new().expect("Failed to initialise codegen engine")));

/// Render a named template with the provided JSON context.
///
/// * `template_name` – name of a template file located in the `templates/` directory.
/// * `context` – JSON value that will be passed to the template engine.
///
/// Returns the rendered string or a `CodegenError`.
pub fn render(template_name: &str, context: &serde_json::Value) -> Result<String> {
    GLOBAL_ENGINE.render(template_name, context)
}

/// Render a raw template string (Handlebars syntax) with the provided JSON context.
pub fn render_raw(template: &str, context: &serde_json::Value) -> Result<String> {
    GLOBAL_ENGINE.render_raw(template, context)
}

/// Convenience helper that renders a template and writes the result to `output_path`.
///
/// * `template_name` – name of the template.
/// * `context` – JSON data.
/// * `output_path` – destination file path.
pub fn generate_and_write(
    template_name: &str,
    context: &serde_json::Value,
    output_path: impl AsRef<std::path::Path>,
) -> Result<()> {
    let rendered = render(template_name, context)?;
    write_to(output_path, &rendered)
}
