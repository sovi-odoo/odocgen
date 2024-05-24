use std::{collections::HashMap, fs::File, io::Write, rc::Rc};

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
    pub args: Vec<String>,
    pub var_arg: Option<String>,
    pub kw_only_args: Vec<String>,
    pub kw_arg: Option<String>,

    pub doc_string: Option<String>,
}

#[derive(Debug, Default)]
struct FieldData {
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
                write!(html, r##"<details><summary id="f-{f_name}">{f_name}</summary>"##)?;
                write!(html, r##"<pre>{}</pre></details>"##, f_data.declaration)?;
            }
        }

        if !self.methods.is_empty() {
            write!(html, "<h3>Methods</h3>")?;
            for (m_name, m_data) in self.methods.sorted_iter() {
                let m_str = m_data.to_string();
                
                if let Some(doc_string) = &m_data.doc_string {
                    write!(html, r##"<details><summary id="m-{m_name}">{m_name}({m_str})</summary>"##)?;
                    write!(html, r##"<pre>{doc_string}</pre></details>"##)?;
                } else {
                    write!(html, r##"<ul id="m-{m_name}"><li>{m_name}({m_str})</li></ul>"##)?;
                }
            }
        }

        Ok(())
    }
}

fn main() {
    let mut state = State::default();
    for filename in std::env::args().skip(1) {
        let code = std::fs::read_to_string(&filename).unwrap();
        let result = rustpython_parser::parse(&code, rustpython_parser::Mode::Module, &filename).unwrap();

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
                                    field_data.declaration = code[stmt.range.start().to_usize()..stmt.range.end().to_usize()].to_string();
                                    local_data.fields.insert(name.to_string(), field_data);
                                } else if name == "_name" {
                                    if let Some(class_name) = stmt.value.clone().constant_expr().map(|x| x.value.str()).flatten() {
                                        class_names.push(class_name);
                                    }
                                } else if name == "_inherit" {
                                    if let Some(class_name) = stmt.value.clone().constant_expr().map(|x| x.value.str()).flatten() {
                                        class_names.push(class_name);
                                        inherits = true;
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
                    let class_data = state.classes.entry(name).or_default();
                    if inherits {
                        class_data.inherits.push(Rc::clone(&local_data));
                    } else {
                        class_data.original = Some(Rc::clone(&local_data));
                    }
                }
            }
        }
    }

    for c_data in state.classes.values_mut() {
        c_data.inherits.sort_by(|x, y| x.filename.cmp(&y.filename));
    }

    write_output(&state).unwrap();
}

fn write_output(state: &State) -> std::io::Result<()> {
    match std::fs::remove_dir_all("output") {
        Ok(()) => (),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (),
        Err(err) => return Err(err),
    }

    std::fs::create_dir_all("output/class")?;

    std::fs::write("output/index.html", include_bytes!("../static/index.html"))?;
    std::fs::write("output/class/class.js", include_bytes!("../static/class.js"))?;
    std::fs::write("output/class/class.css", include_bytes!("../static/class.css"))?;

    let mut index_js = File::create("output/index.js")?;
    index_js.write_all(b"'use strict'\nconst globalIndex = {\n")?;

    index_js.write_all(b"classes: [\n")?;
    for name in state.classes.sorted_keys() {
        writeln!(&mut index_js, "{name:?},")?;
    }
    index_js.write_all(b"],\n")?;

    index_js.write_all(b"methods: {\n")?;
    for (c_name, c_data) in state.classes.sorted_iter() {
        if let Some(orig) = &c_data.original {
            for m_name in orig.methods.sorted_keys() {
                writeln!(&mut index_js, "{m_name:?}:{{o:true,c:{c_name:?}}},")?;
            }
        }

        for data in c_data.inherits.iter() {
            for m_name in data.methods.keys() {
                writeln!(&mut index_js, "{m_name:?}:{{o:false,c:{c_name:?}}},")?;
            }
        }
    }
    index_js.write_all(b"},\n")?;

    index_js.write_all(b"fields: {\n")?;
    for (c_name, c_data) in state.classes.sorted_iter() {
        if let Some(orig) = &c_data.original {
            for f_name in orig.fields.keys() {
                writeln!(&mut index_js, "{f_name:?}:{{o:true,c:{c_name:?}}},")?;
            }
        }

        for data in c_data.inherits.iter() {
            for f_name in data.fields.keys() {
                writeln!(&mut index_js, "{f_name:?}:{{o:false,c:{c_name:?}}},")?;
            }
        }
    }
    index_js.write_all(b"},\n")?;

    index_js.write_all(b"}\n")?;

    std::mem::drop(index_js);

    for (c_name, c_data) in state.classes.sorted_iter() {
        let mut html = File::create(format!("output/class/{c_name}.html"))?;
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
