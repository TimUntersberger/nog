use interpreter::{AstKind, AstNode, Parser};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug)]
enum DocumentationKind {
    Function,
    Class,
}

#[derive(Debug)]
struct Example {
    source: Vec<String>,
    output: Option<Vec<String>>,
}

#[derive(Debug)]
struct Documentation {
    /// unique path to documented item
    path: String,
    kind: DocumentationKind,
    description: Option<Vec<String>>,
    example: Option<Example>,
    param_docs: BTreeMap<String, String>,
    return_value: Option<String>,
}

impl Documentation {
    pub fn new(kind: DocumentationKind, path: String) -> Self {
        Self {
            path,
            kind,
            description: None,
            example: None,
            param_docs: BTreeMap::new(),
            return_value: None,
        }
    }
}

fn main() {
    let root_dir: PathBuf = [
        std::env::current_dir().unwrap().to_str().unwrap(),
        "nogscript",
    ]
    .iter()
    .collect();

    let mut root_path = root_dir.clone();
    root_path.push("main.ns");

    let source = std::fs::read_to_string(&root_path).unwrap();

    let mut parser = Parser::new();
    parser.set_source(root_path.clone(), &source, 0);

    let prog = parser.parse().clone().unwrap();
    let mut docs = Vec::new();

    parse_docs(&prog.stmts, &root_path, "".into(), &mut docs);

    for doc in &docs {
        println!("{:?} {}", doc.kind, doc.path);
    }

    gen_mdbook(docs);
}

#[derive(Debug)]
enum Content {
    Mod(BTreeMap<String, Content>),
    Doc(Documentation),
}

impl Default for Content {
    fn default() -> Self {
        Self::Mod(BTreeMap::default())
    }
}

impl Content {
    pub fn as_mod(&self) -> Option<&BTreeMap<String, Content>> {
        match self {
            Self::Mod(x) => Some(x),
            _ => None,
        }
    }
    pub fn as_mod_mut(&mut self) -> Option<&mut BTreeMap<String, Content>> {
        match self {
            Self::Mod(x) => Some(x),
            _ => None,
        }
    }
}

fn write_mdbook_sidebar(
    buffer: &mut String,
    depth: usize,
    folder_path: String,
    node: &BTreeMap<String, Content>,
) {
    for (key, val) in node.iter() {
        let indentation = "  ".repeat(depth);
        let item_kind = match &val {
            Content::Mod(_) => "M",
            Content::Doc(doc) => match doc.kind {
                DocumentationKind::Function => "F",
                DocumentationKind::Class => "C",
            },
        };
        let file_path = format!("{}{}.md", folder_path, key);
        let s = format!("{}- [{}]({})\n", indentation, key, file_path);
        buffer.push_str(&s);
        match val {
            Content::Mod(node) => {
                write_mdbook_sidebar(
                    buffer,
                    depth + 1,
                    format!("{}{}/", folder_path, key),
                    val.as_mod().unwrap(),
                );
            }
            Content::Doc(_) => {}
        }
    }
}

fn gen_mdbook_sidebar(dir_path: &PathBuf, root: &BTreeMap<String, Content>) {
    let mut file_path = dir_path.clone();
    file_path.push("SUMMARY.md");

    let mut buffer = String::new();
    buffer.push_str("# API\n");
    buffer.push_str("# API\n");

    write_mdbook_sidebar(&mut buffer, 0, "./api/".into(), &root);

    std::fs::write(file_path, buffer).expect("Failed to write SUMMARY.md");
}

fn write_mdbook_content(folder_path: &mut PathBuf, node: &BTreeMap<String, Content>) {
    for (key, val) in node {
        match val {
            Content::Mod(items) => {
                folder_path.push(key);

                if !folder_path.exists() {
                    std::fs::create_dir(&folder_path).unwrap();
                }

                write_mdbook_content(folder_path, val.as_mod().unwrap());

                folder_path.set_extension("md");

                let mut functions = Vec::new();
                let mut classes = Vec::new();
                let mut modules = Vec::new();

                for (key, val) in items {
                    match val {
                        Content::Mod(_) => {
                            modules.push(key);
                        }
                        Content::Doc(doc) => match doc.kind {
                            DocumentationKind::Function => {
                                functions.push(key);
                            }
                            DocumentationKind::Class => {
                                classes.push(key);
                            }
                        },
                    }
                }

                let mut buffer = String::new();
                buffer.push_str(&format!("# {}", key));
                buffer.push_str("\n");
                buffer.push_str("\n");

                if !modules.is_empty() {
                    buffer.push_str("## Modules");
                    buffer.push_str("\n");

                    for m in modules {
                        buffer.push_str(&format!("* [{}]()", m));
                        buffer.push_str("\n");
                    }
                }

                if !functions.is_empty() {
                    buffer.push_str("## Functions");
                    buffer.push_str("\n");

                    for f in functions {
                        buffer.push_str(&format!("* [{}]()", f));
                        buffer.push_str("\n");
                    }
                }

                if !classes.is_empty() {
                    buffer.push_str("## Classes");
                    buffer.push_str("\n");

                    for c in classes {
                        buffer.push_str(&format!("* [{}]()", c));
                        buffer.push_str("\n");
                    }
                }

                std::fs::write(&folder_path, buffer).unwrap();
                folder_path.pop();
            }
            Content::Doc(doc) => {
                folder_path.push(key);
                folder_path.set_extension("md");
                let mut buffer = String::new();
                buffer.push_str(&format!("# {}", key));
                buffer.push_str("\n");

                match doc.kind {
                    DocumentationKind::Function => {
                        let mut args: Vec<String> = Vec::new();

                        for (key, val) in &doc.param_docs {
                            args.push(format!("{}: {}", key, val));
                        }

                        let ret = doc
                            .return_value
                            .as_ref()
                            .map(|x| x.to_string())
                            .unwrap_or("Void".into());

                        // Description
                        buffer.push_str("\n");
                        if let Some(lines) = &doc.description {
                            for line in lines {
                                buffer.push_str(if !line.is_empty() { &line[1..] } else { line });
                                buffer.push_str("\n");
                            }
                        }

                        // Signature
                        buffer.push_str("## Signature\n");
                        buffer.push_str("\n");
                        buffer.push_str("```nogscript\n");
                        buffer.push_str(&format!("fn {}({}) -> {}\n", key, args.join(", "), ret));
                        buffer.push_str("```\n");
                        buffer.push_str("\n");

                        // Example
                        if let Some(example) = &doc.example {
                            buffer.push_str("## Example\n");
                            buffer.push_str("\n");
                            buffer.push_str("```nogscript\n");
                            for line in &example.source {
                                buffer.push_str(line);
                                buffer.push_str("\n");
                            }
                            buffer.push_str("```\n");
                            buffer.push_str("\n");

                            // Output
                            if let Some(lines) = &example.output {
                                buffer.push_str("Output \n");
                                buffer.push_str("\n");
                                buffer.push_str("```nogscript\n");
                                for line in lines {
                                    buffer.push_str(line);
                                    buffer.push_str("\n");
                                }
                                buffer.push_str("```\n");
                                buffer.push_str("\n");
                            }
                        }
                    }
                    DocumentationKind::Class => {}
                }

                std::fs::write(&folder_path, buffer).unwrap();
                folder_path.pop();
            }
        }
    }
}

fn gen_mdbook_content(dir_path: &PathBuf, root: &BTreeMap<String, Content>) {
    let mut api_path = dir_path.clone();
    api_path.push("api");

    if !api_path.exists() {
        std::fs::create_dir(&api_path).expect("Failed to create api directory");
    }

    write_mdbook_content(&mut api_path, &root);
}

fn gen_mdbook(docs: Vec<Documentation>) {
    let root_dir: PathBuf = [std::env::current_dir().unwrap().to_str().unwrap(), "output"]
        .iter()
        .collect();

    if !root_dir.exists() {
        std::fs::create_dir(&root_dir).expect("Failed to create output directory");
    }

    let mut root_node: BTreeMap<String, Content> = BTreeMap::new();

    for doc in docs {
        let parts = doc.path.split(".").collect::<Vec<_>>();
        let mut map_ref = &mut root_node;

        for i in 0..parts.len() - 1 {
            map_ref = map_ref
                .entry(parts[i].to_string())
                .or_default()
                .as_mod_mut()
                .unwrap();
        }

        map_ref.insert(parts.last().unwrap().to_string(), Content::Doc(doc));
    }

    gen_mdbook_sidebar(&root_dir, &root_node);
    gen_mdbook_content(&root_dir, &root_node);
}

fn parse_docs(
    stmts: &Vec<AstNode>,
    root_path: &PathBuf,
    current_mod_path: String,
    docs: &mut Vec<Documentation>,
) {
    let mut iter = stmts.iter().peekable();
    while let Some(stmt) = iter.next() {
        match &stmt.kind {
            AstKind::ImportStatement(path) => {
                let parts = path.split(".");
                let mut mod_path = root_path.clone();

                mod_path.pop();

                for part in parts {
                    mod_path.push(part);
                }

                mod_path.set_extension("ns");

                if mod_path.exists() {
                    let source = std::fs::read_to_string(&mod_path).unwrap();
                    let mut parser = Parser::new();
                    parser.set_source(mod_path.clone(), &source, 0);
                    let prog = parser.parse().unwrap();

                    parse_docs(&prog.stmts, &mod_path, path.clone(), docs);
                }
            }
            AstKind::Documentation(lines) => {
                if let Some(mut item) = iter.next() {
                    match &item.kind {
                        AstKind::ExportStatement(x) => {
                            item = x.as_ref();
                        }
                        AstKind::ExternStatement(x) => {
                            item = x.as_ref();
                        }
                        _ => {}
                    }
                    let (kind, mut name) = match &item.kind {
                        AstKind::FunctionDefinition(name, _, _) => {
                            (DocumentationKind::Function, name.clone())
                        }
                        AstKind::ClassDefinition(name, _) => {
                            (DocumentationKind::Class, name.clone())
                        }
                        x => todo!("{:?}", x),
                    };

                    if !current_mod_path.is_empty() {
                        name = format!("{}.{}", current_mod_path, name);
                    }

                    let mut doc = Documentation::new(kind, name.clone());

                    let mut lines_iter = lines.iter().peekable();
                    let mut description = Vec::new();

                    while let Some(line) = lines_iter.peek() {
                        if line.starts_with(" @") {
                            break;
                        }

                        description.push(lines_iter.next().unwrap().clone());
                    }

                    doc.description = Some(description);

                    while let Some(line) = lines_iter.peek() {
                        if line.starts_with(" @param") {
                            let parts = line
                                .split(" ")
                                .filter(|x| !x.is_empty())
                                .collect::<Vec<_>>();
                            let arg_name = parts[1].to_string();
                            let arg_type = parts[2].to_string();
                            doc.param_docs.insert(arg_name, arg_type);
                        } else if line.starts_with(" @returns") {
                            let parts = line
                                .split(" ")
                                .filter(|x| !x.is_empty())
                                .collect::<Vec<_>>();
                            doc.return_value = Some(parts[1].to_string());
                        } else if line.starts_with(" @example") {
                            let mut is_output = false;
                            let mut output = Vec::new();
                            let mut source = Vec::new();

                            lines_iter.next();

                            while let Some(line) = lines_iter.next() {
                                if line.starts_with(" @example") {
                                    break;
                                } else if line.starts_with(" @output") {
                                    is_output = true;
                                    continue;
                                }

                                if is_output {
                                    output.push(line.to_string());
                                } else {
                                    source.push(line.to_string());
                                }
                            }

                            doc.example = Some(Example {
                                source,
                                output: if is_output { Some(output) } else { None },
                            })
                        }

                        lines_iter.next();
                    }

                    docs.push(doc);
                }
            }
            _ => {}
        }
    }
}
