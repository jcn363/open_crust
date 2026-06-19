//! Template engine wrapper supporting Tera and Handlebars.

use anyhow::Context;
use handlebars::Handlebars;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tera::{Context as TeraContext, Tera};

use crate::errors::Result;

/// Central engine that can render using Tera (full-featured) and Handlebars (simple).
pub struct Engine {
    tera: Option<Tera>,
    handlebars: Handlebars<'static>,
}

impl Engine {
    /// Initialise the engine.
    ///
    /// * Loads all ``*.tera`` files from the ``templates/`` directory into a Tera instance.
    /// * Registers all ``*.hbs`` files as Handlebars templates.
    pub fn new() -> Result<Self> {
        // Load Tera templates if the directory exists.
        let tera_dir = Path::new("templates");
        let tera = if tera_dir.is_dir() {
            let pattern = format!("{}/**/*.tera", tera_dir.display());
            let mut t = Tera::new(&pattern).with_context(|| {
                format!("Failed to parse Tera templates in {}", tera_dir.display())
            })?;
            // Enable autoescaping for safety.
            t.autoescape_on(vec!["html", "htm", "xml"]);
            Some(t)
        } else {
            None
        };

        // Initialise Handlebars and register any ``*.hbs`` files.
        let mut handlebars = Handlebars::new();
        let hbs_dir = Path::new("templates");
        if hbs_dir.is_dir() {
            for entry in std::fs::read_dir(hbs_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("hbs") {
                    let name = path
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .ok_or_else(|| anyhow::anyhow!("Invalid template filename"))?;
                    let tmpl = std::fs::read_to_string(&path)?;
                    handlebars.register_template_string(name, tmpl)?;
                }
            }
        }

        Ok(Self { tera, handlebars })
    }

    /// Render a named template with the given JSON context.
    ///
    /// Tries Tera first; if the template is not found, falls back to Handlebars.
    pub fn render(&self, name: &str, ctx: &Value) -> Result<String> {
        if let Some(tera) = &self.tera {
            if tera.get_template(name).is_ok() {
                let mut tera_ctx = TeraContext::new();
                // Tera expects a map; we can insert the whole JSON as a variable.
                tera_ctx.insert("data", ctx);
                return Ok(tera.render(name, &tera_ctx)?);
            }
        }
        // Fallback to Handlebars.
        self.handlebars.render(name, ctx).map_err(|e| e.into())
    }

    /// Render a raw template string (Handlebars syntax) with the given JSON context.
    pub fn render_raw(&self, template: &str, ctx: &Value) -> Result<String> {
        // Handlebars is used for raw strings because it can compile on‑the‑fly.
        self.handlebars.render_template(template, ctx).map_err(|e| e.into())
    }
}
