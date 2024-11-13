use std::fmt;
use std::fs;
use std::io;
use std::path::PathBuf;


#[derive(clap::ValueEnum, Clone)]
pub enum FontSize {
    Tiny,
    Scriptsize,
    Footnotesize,
    Small,
    Normalsize,
    Large,
}

impl fmt::Display for FontSize {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use FontSize::*;
        match self {
            Tiny => write!(f, "tiny"),
            Scriptsize => write!(f, "scriptsize"),
            Footnotesize => write!(f, "footnotesize"),
            Small => write!(f, "small"),
            Normalsize => write!(f, "normalsize"),
            Large => write!(f, "large"),
        }
    }
}

fn html_start() -> String {
    let mut html_start_string = "".to_string();
    html_start_string.push_str("<!DOCTYPE html>\n");
    html_start_string.push_str("<html lang=\"en\">\n");
    html_start_string.push_str("<head>\n");
    html_start_string.push_str("    <meta charset=\"UTF-8\">\n");
    html_start_string.push_str("    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html_start_string.push_str("    <title>Code Display</title>\n");
    html_start_string.push_str("    <link rel=\"stylesheet\" href=\"style.css\">\n");
    html_start_string.push_str("</head>\n");
    html_start_string
}

fn html_end() -> String {
    "</html>\n".to_string()
}

#[derive(clap::Args)]
pub struct Args {
    #[clap(value_parser, value_name = "FILE")]
    filepath: PathBuf,
    #[clap(long, default_value_t = 80)]
    width: usize,
    #[clap(long, default_value_t=FontSize::Scriptsize)]
    fontsize: FontSize,
    #[clap(long, num_args = 0)]
    omit_lambda_sugar: bool,
    #[clap(long, num_args = 0)]
    omit_function_sugar: bool,
    #[clap(long, default_value_t = 4)]
    indent: isize,
    #[clap(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
}

fn compute_output_stream(cmd: &Args) -> Box<dyn io::Write> {
    let mut fp = cmd.filepath.clone();
    fp.pop(); // Remove the file name
    fp.push("docs"); // Add the "doc" directory
    fs::create_dir_all(&fp).expect("Failed to create doc directory");
    fp.push("foo.html"); // Add the "foo.html" file
    Box::new(fs::File::create(fp).expect("Failed to create file"))
}

fn write_template(stream: &mut dyn io::Write) -> io::Result<()> {
    let mut template = "".to_string();
    template.push_str(&html_start());
    template.push_str("<body>\n");
    template.push_str("    <div>\n");
    template.push_str("        <h1>Code Example</h1>\n");
    template.push_str("        <pre><code>\n");
    template.push_str("<span class=\"keyword\">data</span> Nat { Z, S(n: Nat) }\n\n");
    template.push_str("<span class=\"keyword\">data</span> NotZero(n: Nat) {\n");
    template.push_str("    SNotZero(n: Nat): NotZero(S(n))\n");
    template.push_str("}\n\n");
    template.push_str("<span class=\"keyword\">def</span> NotZero(Z).elim_zero(a: Type): a { SNotZero(n) absurd }\n\n");
    template.push_str("<span class=\"keyword\">data</span> Bot { }\n\n");
    template.push_str("<span class=\"keyword\">data</span> Foo(a: Type) {\n");
    template.push_str("    Ok(a: Type, x: a): Foo(a),\n");
    template.push_str("    Absurd(x: NotZero(Z)): Foo(Bot)\n");
    template.push_str("}\n\n");
    template.push_str("<span class=\"keyword\">def</span> Foo(a).elim(a: Type): a {\n");
    template.push_str("    Ok(a, x) =&gt; x,\n");
    template.push_str("    Absurd(x) =&gt; x.elim_zero(Bot)\n");
    template.push_str("}\n");
    template.push_str("        </code></pre>\n");
    template.push_str("    </div>\n");
    template.push_str("</body>\n");
    template.push_str(&html_end());
    
    stream.write_all(template.as_bytes())
}

pub fn exec(cmd: Args) -> miette::Result<()> {

    let mut stream: Box<dyn io::Write> = compute_output_stream(&cmd);

    write_template(&mut stream).expect("Failed to write template");
    Ok(())
}
