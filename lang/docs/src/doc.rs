use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

use askama::Template;
use opener;

use driver::paths::{CSS_PATH, CSS_TEMPLATE_PATH};
use driver::Database;
use printer::{Print, PrintCfg};

use crate::generate_docs::GenerateDocs;

const HTML_END: &str = " </code></pre>
    </div></body></html>";

fn html_start(filepath: &Path) -> String {
    format!(
        "<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>{filename}</title>
    <link rel=\"stylesheet\" href=\"style.css\">
</head>
<body>
<div>
        <h1>{filename}</h1>
        <pre><code>",
        filename = filepath.file_name().unwrap().to_string_lossy()
    )
}

pub async fn write_html(filepath: &PathBuf, htmlpath: &PathBuf) {
    let mut db = Database::from_path(filepath);
    let uri = db.resolve_path(filepath).expect("Failed to resolve path");
    let prg = db.ust(&uri).await.expect("Failed to get UST");
    let cfg = PrintCfg::default();

    if !Path::new(CSS_PATH).exists() {
        fs::create_dir_all(Path::new(CSS_PATH).parent().unwrap())
            .expect("Failed to create CSS directory");
        fs::write(CSS_PATH, CSS_TEMPLATE_PATH).expect("Failed to create CSS file");
    }

    let mut stream = Box::new(fs::File::create(htmlpath).expect("Failed to create file"));
    stream.write_all(html_start(filepath).as_bytes()).expect("Failed to write to file");
    print_prg(&prg, &cfg, &mut stream);
    stream.write_all(HTML_END.as_bytes()).expect("Failed to write to file");
    println!("new Generate: {}", prg.generate_docs())
}

fn print_prg<W: io::Write>(prg: &ast::Module, cfg: &PrintCfg, stream: &mut W) {
    prg.print_html(cfg, stream).expect("Failed to print to stdout");
    println!();
}

pub fn open(filepath: &PathBuf) {
    let absolute_path = fs::canonicalize(filepath).expect("Failed to get absolute path");
    opener::open(&absolute_path).unwrap();
}

#[derive(Template)]
#[template(path = "code.html", escape = "none")]
struct HelloTemplate<'a> {
    title: &'a str,
    code: &'a str,
}

fn generate_html(title: &str, code: &str) -> String {
    let template = HelloTemplate { title, code };
    template.render().unwrap()
}
