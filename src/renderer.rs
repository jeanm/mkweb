use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use handlebars::Handlebars;
use pandoc;
use toml;
use walkdir::WalkDir;
use error::{Error, Result};

pub fn read_frontmatter_and_content(
    raw: String,
) -> Result<(Option<toml::Table>, String)> {
    if raw.starts_with("+++\n") {
        match raw[4..].find("\n+++\n") {
            Some(end) => {
                Ok((
                    toml::decode_str(&raw[4..end+4]),
                    raw[end+4+5..].to_string(),
                ))
            }
            None => Ok((None, raw)),
        }
    } else {
        Ok((None, raw))
    }
}

pub struct Renderer {
    root_path: PathBuf,
    public_path: PathBuf,
    assets_path: PathBuf,
    handlebars: Handlebars,
}

impl Renderer {
    pub fn new(root_path: &str) -> Result<Renderer> {
        let root_path = PathBuf::from(root_path);
        let public_path = root_path.join("public/");
        let assets_path = root_path.join("assets/");
        fs::create_dir_all(&public_path)?;

        let mut handlebars = Handlebars::new();
        let mut template_file = File::open(root_path.join("template.hbs"))?;
        let mut template_contents = String::new();
        template_file.read_to_string(&mut template_contents)?;
        handlebars.register_template_string("template", template_contents)?;

        Ok(Renderer {
            root_path: root_path,
            public_path: public_path,
            assets_path: assets_path,
            handlebars: handlebars,
        })
    }

    // TODO only render files after checking timestamps
    pub fn render(&self) -> Result<()> {
        self.copy_assets()?;
        let walk = WalkDir::new(&self.root_path);
        let paths: Vec<_> = walk
            .into_iter()
            .filter_map(|e| e.ok())
            .map(|e| e.path().to_owned()).collect();
        for epath in paths {
            if epath.starts_with(&self.public_path) ||
               epath.starts_with(&self.assets_path) {
                continue;
            }
            let relative_epath = epath.strip_prefix(&self.root_path).unwrap();
            let public_epath = self.public_path.join(relative_epath);
            if epath.is_dir() {
                fs::create_dir_all(&public_epath)?;
                continue;
            }
            // skip files if unchanged
            let modified = epath.metadata()?.modified()?;
            if public_epath.exists() {
                let public_modified = public_epath.metadata()?.modified()?;
                if public_modified >= modified {
                    println!(
                        "{} is not older than source: skipping.",
                        public_epath.display()
                    );
                    continue
                }
            }
            match epath.extension() {
                None => continue,
                Some(ext) => if ext != "html" && ext != "md" { continue },
            }

            let rendered = self.render_file(&epath)?;
            let mut output_file = File::create(public_epath)?;
            writeln!(output_file, "{}", rendered)?;
        }
        Ok(())
    }

    // TODO only copy changed assets
    fn copy_assets(&self) -> Result<()> {
        let walk = WalkDir::new(&self.assets_path);
        for entry in walk.into_iter().filter_map(|e| e.ok()) {
            let apath = entry.path();
            let relative_apath = apath.strip_prefix(&self.assets_path).unwrap();
            let public_apath = self.public_path.join(relative_apath);
            if apath.is_dir() {
                fs::create_dir_all(&public_apath)?;
                continue;
            }
            // skip files if unchanged
            let modified = apath.metadata()?.modified()?;
            if public_apath.exists() {
                let public_modified = public_apath.metadata()?.modified()?;
                if public_modified >= modified {
                    println!(
                        "{} is not older than source: skipping.",
                        public_apath.display()
                    );
                    continue
                }
            }
            if apath.is_file() {
                fs::copy(apath, public_apath)?;
            }
        }
        Ok(())
    }

    fn render_file(&self, path: &Path) -> Result<String> {
        match path.extension() {
            None => Err(Error::UnrecognisedExtension("".to_string())),
            Some(ext) => {
                if ext == "md" || ext == "html" {
                    let mut file = File::open(path)?;
                    let mut raw = String::new();
                    file.read_to_string(&mut raw)?;
                    let (frontmatter, content) =
                        read_frontmatter_and_content(raw)?;
                    if ext == "md" {
                        self.render_md(frontmatter, content)
                    } else {
                        self.render_html(frontmatter, content)
                    }
                } else {
                    let ext = ext.to_string_lossy().to_string();
                    Err(Error::UnrecognisedExtension(ext))
                }
            }
        }
    }

    fn render_html(
        &self,
        frontmatter: Option<toml::Table>,
        content: String,
    ) -> Result<String> {
        let mut data = match frontmatter {
            Some(f) => f,
            None => BTreeMap::new(),
        };
        data.insert(
            "content".to_string(),
            toml::Value::String(content)
        );
        Ok(self.handlebars.render("template", &data)?)
    }

    fn render_md(
        &self,
        frontmatter: Option<toml::Table>,
        content: String,
    ) -> Result<String> {
        let mut pandoc = pandoc::new();
        pandoc.set_output_format(pandoc::OutputFormat::Html5)
                .set_output(pandoc::OutputKind::Pipe)
                .set_input_format(pandoc::InputFormat::Markdown)
                .set_input(pandoc::InputKind::Pipe(content))
                .add_option(pandoc::PandocOption::MathJax(None));
        let pandoc_output = pandoc.execute()?;

        let content = match pandoc_output {
            pandoc::PandocOutput::ToBuffer(s) => s,
            _ => unreachable!(),
        };

        self.render_html(frontmatter, content)
    }
}