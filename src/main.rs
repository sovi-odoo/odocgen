use std::{collections::HashMap, fs::File, io::Write, path::Path, rc::Rc};
use clap::Parser;
use rustpython_parser::ast::{Ranged, TextSize};

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Directory to write output in (deleted if exists)
    #[arg(long, short)]
    output: String,

    /// Name of the branch the documentation is generated for
    #[arg(long, short)]
    branch: String,

    /// Directories to find addons in (e.g. "odoo/addons")
    addons_dirs: Vec<String>,
}

#[derive(Debug, Default)]
struct State {
    pub classes: HashMap<String, ClassData>,
}

#[derive(Debug, Default)]
struct ClassData {
    pub original: Option<Rc<LocalData>>,
    pub inherits: Vec<Rc<LocalData>>,
}

#[derive(Debug, Default)]
struct LocalData {
    pub filename: Rc<String>,
    pub methods: HashMap<String, MethodData>,
    pub fields: HashMap<String, FieldData>,
}

#[derive(Debug, Default)]
struct MethodData {
    pub line_col: (usize, usize),

    pub args: Vec<String>,
    pub var_arg: Option<String>,
    pub kw_only_args: Vec<String>,
    pub kw_arg: Option<String>,

    pub doc_string: Option<String>,
}

#[derive(Debug, Default)]
struct FieldData {
    pub line_col: (usize, usize),

    pub declaration: String,
}

impl ToString for MethodData {
    fn to_string(&self) -> String {
        let mut m_str = self.args.join(", ");

        if let Some(x) = &self.var_arg {
            if !m_str.is_empty() {
                m_str.push_str(", ");
            }

            m_str.push('*');
            m_str.push_str(x);
        }

        let kw_only = self.kw_only_args.join(", ");
        if !kw_only.is_empty() {
            if !m_str.is_empty() {
                m_str.push_str(", ");
            }

            m_str.push_str(&kw_only);
        }

        if let Some(x) = &self.kw_arg {
            if !m_str.is_empty() {
                m_str.push_str(", ");
            }

            m_str.push_str("**");
            m_str.push_str(x);
        }

        m_str
    }
}

impl LocalData {
    pub fn write_to_html(&self, html: &mut File, title: &str) -> std::io::Result<()> {
        if self.fields.is_empty() && self.methods.is_empty() { return Ok(()) }

        write!(html, "<h2>{title}: {}</h2>", self.filename)?;

        if !self.fields.is_empty() {
            write!(html, "<h3>Fields</h3>")?;
            for (f_name, f_data) in self.fields.sorted_iter() {
                let (line, _) = f_data.line_col;
                write!(html, r##"<details><summary id="f-{f_name}">{f_name} <span class="position">@ line {line}</span></summary>"##)?;
                write!(html, r##"<pre>{}</pre></details>"##, f_data.declaration)?;
            }
        }

        if !self.methods.is_empty() {
            write!(html, "<h3>Methods</h3>")?;
            for (m_name, m_data) in self.methods.sorted_iter() {
                let m_str = m_data.to_string();
                let (line, _) = m_data.line_col;
                
                if let Some(doc_string) = &m_data.doc_string {
                    write!(html, r##"<details><summary id="m-{m_name}">{m_name}({m_str}) <span class="position">@ line {line}</span></summary>"##)?;
                    write!(html, r##"<pre>{doc_string}</pre></details>"##)?;
                } else {
                    write!(html, r##"<ul id="m-{m_name}"><li>{m_name}({m_str}) <span class="position">@ line {line}</span></li></ul>"##)?;
                }
            }
        }

        Ok(())
    }
}

impl State {
    fn parse_file(&mut self, filename: String) -> Result<(), Box<dyn std::error::Error>> {
        let code = std::fs::read_to_string(&filename)?;
        let result = rustpython_parser::parse(&code, rustpython_parser::Mode::Module, &filename)?;
        let line_data = parse_line_data(&code);

        let filename_rc = Rc::new(filename);

        for stmt in result.module().unwrap().body {
            if let Some(stmt) = stmt.class_def_stmt() {
                let mut class_names = Vec::new();
                let mut local_data = LocalData {
                    filename: Rc::clone(&filename_rc),
                    ..Default::default()
                };
                let mut inherits = false;

                'class_body: for stmt in stmt.body.iter() {
                    if let Some(stmt) = stmt.as_assign_stmt() {
                        for target in stmt.targets.iter() {
                            if let Some(name) = target.as_name_expr() {
                                let name = &name.id;
                                if !name.starts_with('_') {
                                    let mut field_data = FieldData::default();
                                    field_data.line_col = find_line_col(&line_data, stmt.start());
                                    field_data.declaration = code[stmt.range.start().to_usize()..stmt.range.end().to_usize()].to_string();
                                    local_data.fields.insert(name.to_string(), field_data);
                                } else if name == "_name" {
                                    if let Some(class_name) = stmt.value.clone().constant_expr().map(|x| x.value.str()).flatten() {
                                        class_names.push(class_name);
                                    }
                                } else if name == "_inherit" || name == "_inherits" {
                                    inherits = true;

                                    if let Some(class_name) = stmt.value.clone().constant_expr().map(|x| x.value.str()).flatten() {
                                        class_names.push(class_name);
                                    }

                                    if let Some(inherits_list) = stmt.value.clone().list_expr() {
                                        for element in inherits_list.elts {
                                            if let Some(class_name) = element.clone().constant_expr().map(|x| x.value.str()).flatten() {
                                                let _ = class_name;
                                                // Broken
                                                // class_names.push(class_name);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }

                    if let Some(stmt) = stmt.as_function_def_stmt() {
                        for decorator in stmt.decorator_list.iter() {
                            let skip = if let Some(call) = decorator.as_call_expr() {
                                call.func
                                    .as_attribute_expr()
                                    .map(|attr| attr.value.as_name_expr().map(|name| &name.id == "api"))
                                    .flatten()
                                    .unwrap_or(false)
                            } else if let Some(attr) = decorator.as_attribute_expr() {
                                &attr.attr == "model" || attr.value.as_name_expr().map(|name| &name.id == "api").unwrap_or(false)
                            } else {
                                false
                            };

                            if skip {
                                continue 'class_body
                            }
                        }

                        let name = stmt.name.to_string();
                        let mut method_data = MethodData::default();
                        method_data.line_col = find_line_col(&line_data, stmt.start());

                        method_data.args = stmt.args.args
                            .iter()
                            .map(|arg| arg.def.arg.to_string())
                            .collect();

                        method_data.var_arg = stmt.args.vararg
                            .as_ref()
                            .map(|arg| arg.arg.to_string());

                        method_data.kw_only_args = stmt.args.kwonlyargs
                            .iter()
                            .map(|arg| arg.def.arg.to_string())
                            .collect();

                        method_data.kw_arg = stmt.args.vararg
                            .as_ref()
                            .map(|arg| arg.arg.to_string());

                        method_data.doc_string = stmt.body
                            .first()
                            .map(|stmt| stmt
                                .as_expr_stmt()
                                .map(|expr| expr.value
                                    .as_constant_expr()
                                    .map(|expr| expr.value
                                        .as_str()
                                        .map(|string| format_doc_string(string))
                                    )
                                )
                            )
                            .flatten()
                            .flatten()
                            .flatten();

                        local_data.methods.insert(name, method_data);
                    }
                }

                let local_data = Rc::new(local_data);
                for name in class_names {
                    let class_data = self.classes.entry(name).or_default();
                    if inherits {
                        class_data.inherits.push(Rc::clone(&local_data));
                    } else {
                        class_data.original = Some(Rc::clone(&local_data));
                    }
                }
            }
        }

        Ok(())
    }
}

fn parse_line_data(code: &str) -> Vec<usize> {
    let mut result = Vec::new();
    for (i, b) in code.bytes().enumerate() {
        if b == b'\n' {
            result.push(i);
        }
    }

    result
}

fn find_line_col(line_data: &[usize], position: TextSize) -> (usize, usize) {
    let position = position.to_usize();
    for (i, &step) in line_data.iter().enumerate() {
        if step > position {
            match line_data.get(i - 1) {
                Some(&last_step) => return (i + 1, position - last_step),
                None => return (1, position + 1),
            }
        }
    }

    match line_data.last() {
        Some(&step) => (line_data.len(), position - step),
        None => (1, position + 1),
    }
}

fn main() {
    let cli = Cli::parse();
    let mut state = State::default();
    for addons_path in cli.addons_dirs {
        for addons_entry in std::fs::read_dir(&addons_path).unwrap() {
            let models_path = addons_entry.unwrap().path().join("models");
            match std::fs::metadata(&models_path) {
                Ok(metadata) if metadata.is_dir() => (),
                _ => continue,
            }
            
            for models_entry in std::fs::read_dir(&models_path).unwrap() {
                let models_entry = models_entry.unwrap().path().to_str().unwrap().to_string();
                if models_entry.ends_with(".py") {
                    state.parse_file(models_entry).unwrap();
                }
            }
        }
    }

    for c_data in state.classes.values_mut() {
        c_data.inherits.sort_by(|x, y| x.filename.cmp(&y.filename));
    }

    let output = Path::new(&cli.output).to_str().unwrap();
    write_output(&state, output, &cli.branch).unwrap();
}

fn write_output(state: &State, output: &str, branch: &str) -> std::io::Result<()> {
    match std::fs::remove_dir_all(output) {
        Ok(()) => (),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
        Err(err) => return Err(err),
    }

    std::fs::create_dir_all(format!("{output}/class"))?;

    std::fs::write(format!("{output}/index.js"), include_bytes!("../embeded/index.js"))?;
    std::fs::write(format!("{output}/class/class.js"), include_bytes!("../embeded/class.js"))?;
    std::fs::write(format!("{output}/class/class.css"), include_bytes!("../embeded/class.css"))?;

    let index_html = include_str!("../embeded/index.html").replace("{{branch}}", &format!("[{branch}]"));
    std::fs::write(format!("{output}/index.html"), index_html.as_bytes())?;
    std::mem::drop(index_html);

    let quotes = include_str!("../embeded/quotes.list")
        .split('\n')
        .map(|x| x.trim())
        .filter(|x| !x.is_empty() && !x.starts_with('#'));

    let mut db_js = File::create(format!("{output}/db.js"))?;
    db_js.write_all(b"'use strict'\nconst globalIndex={")?;

    db_js.write_all(b"classes:[")?;
    for name in state.classes.sorted_keys() {
        write!(&mut db_js, "{name:?},")?;
    }
    db_js.write_all(b"],")?;

    db_js.write_all(b"methods:{")?;
    for (c_name, c_data) in state.classes.sorted_iter() {
        if let Some(orig) = &c_data.original {
            for m_name in orig.methods.sorted_keys() {
                write!(&mut db_js, "{m_name:?}:{{o:true,c:{c_name:?}}},")?;
            }
        }

        for data in c_data.inherits.iter() {
            for m_name in data.methods.keys() {
                write!(&mut db_js, "{m_name:?}:{{o:false,c:{c_name:?}}},")?;
            }
        }
    }
    db_js.write_all(b"},")?;

    db_js.write_all(b"fields:{")?;
    for (c_name, c_data) in state.classes.sorted_iter() {
        if let Some(orig) = &c_data.original {
            for f_name in orig.fields.keys() {
                write!(&mut db_js, "{f_name:?}:{{o:true,c:{c_name:?}}},")?;
            }
        }

        for data in c_data.inherits.iter() {
            for f_name in data.fields.keys() {
                write!(&mut db_js, "{f_name:?}:{{o:false,c:{c_name:?}}},")?;
            }
        }
    }
    db_js.write_all(b"}\n")?;

    db_js.write_all(b"};const globalQuoteList=[")?;
    for quote in quotes {
        write!(&mut db_js, "{quote:?},")?;
    }
    db_js.write_all(b"]")?;

    std::mem::drop(db_js);

    for (c_name, c_data) in state.classes.sorted_iter() {
        let mut html = File::create(format!("{output}/class/{c_name}.html"))?;
        html.write_all(concat!(
            r##"<!doctype html>"##,
            r##"<html lang="en">"##,
            r##"<head>"##,
            r##"<meta charset="UTF-8" />"##,
            r##"<meta name="viewport" content="width=device-width, initial-scale=1.0" />"##,
        ).as_bytes())?;
        write!(&mut html, "<title>{c_name} - odocgen</title>")?;
        html.write_all(concat!(
            r##"<link rel="stylesheet" href="class.css" />"##,
            r##"</head>"##,
            r##"<body>"##,
        ).as_bytes())?;

        write!(&mut html, "<h1>{c_name}</h1>")?;

        if let Some(orig) = &c_data.original {
            write!(&mut html, "<p>Originally defined in: {}</p>", orig.filename)?;
        }

        if !c_data.inherits.is_empty() {
            html.write_all(b"<p>")?;
            for inherits in c_data.inherits.iter().map(|x| Rc::clone(&x.filename)).rev() {
                write!(&mut html, "Inherited in: {inherits}<br/>")?;
            }
            html.write_all(b"</p>")?;
        }

        html.write_all(b"<hr/>")?;
        if let Some(orig) = &c_data.original {
            orig.write_to_html(&mut html, "Original")?;
        }
        for other in c_data.inherits.iter().rev() {
            other.write_to_html(&mut html, "Inherited")?;
        }

        writeln!(&mut html, r##"<script src="class.js"></script></body></html>"##)?;
    }

    Ok(())
}

fn format_doc_string(doc: &str) -> String {
    let mut result: String = doc
        .trim()
        .split('\n')
        .map(|line| line.trim().to_owned() + "\n")
        .collect();

    result.pop();
    result
}

trait HashMapExt {
    type Key;
    type Value;

    fn sorted_iter(&self) -> Vec<(&Self::Key, &Self::Value)>;
    fn sorted_keys(&self) -> Vec<&Self::Key>;
}

impl<K: Ord, V> HashMapExt for HashMap<K, V> {
    type Key = K;
    type Value = V;

    fn sorted_iter(&self) -> Vec<(&Self::Key, &Self::Value)> {
        let mut result: Vec<_> = self.iter().collect();
        result.sort_by_key(|(key, _)| *key);
        result
    }

    fn sorted_keys(&self) -> Vec<&Self::Key> {
        let mut result: Vec<_> = self.keys().collect();
        result.sort_by_key(|key| *key);
        result
    }
}
