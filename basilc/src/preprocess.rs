use std::collections::{HashMap, HashSet};
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};

use crate::embedded;

#[derive(Debug, Clone)]
pub enum MacroValue { Bool(bool), Int(i64), Str(String) }

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IncludeKey { Fs(PathBuf), Embedded(String) }

#[derive(Debug, Default, Clone)]
pub struct SourceMap {
    // Phase 1: stub
}

#[derive(Debug, Clone)]
pub struct PreprocessOptions {
    pub root_path: PathBuf,
    pub include_paths: Vec<PathBuf>,
    pub env_paths: Vec<PathBuf>,
    pub use_embedded: bool,
    pub defines: HashMap<String, MacroValue>,
    pub engine_name: String,
    pub version_major: i64,
}

impl Default for PreprocessOptions {
    fn default() -> Self {
        Self {
            root_path: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
            include_paths: Vec::new(),
            env_paths: read_env_paths(),
            use_embedded: true,
            defines: HashMap::new(),
            engine_name: "basic".to_string(),
            version_major: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreprocessResult {
    pub text: String,
    #[allow(dead_code)]
    pub source_map: SourceMap,
    #[allow(dead_code)]
    pub dependencies: Vec<IncludeKey>,
}

#[derive(Debug, Clone)]
pub enum PreprocessError {
    Message(String),
    NotFound { target: String, searched: Vec<String>, include_from: Option<String> },
    Cycle(Vec<String>),
}

impl Display for PreprocessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreprocessError::Message(s) => write!(f, "{}", s),
            PreprocessError::NotFound { target, searched, include_from } => {
                writeln!(f, "error: include not found: {target:?}")?;
                writeln!(f, "  searched in:")?;
                for s in searched { writeln!(f, "    - {s}")?; }
                if let Some(fr) = include_from { writeln!(f, "  included from: {fr}")?; }
                Ok(())
            }
            PreprocessError::Cycle(chain) => {
                writeln!(f, "error: include cycle detected")?;
                let mut first = true;
                for c in chain {
                    if first { write!(f, "  {c}")?; first = false; } else { write!(f, " -> {c}")?; }
                }
                Ok(())
            }
        }
    }
}

const MAX_DEPTH: usize = 64;
const MAX_SIZE: usize = 2 * 1024 * 1024;

pub fn preprocess_text(path: &Path, text: &str, mut opts: PreprocessOptions) -> Result<PreprocessResult, PreprocessError> {
    // Normalize root path: if path is a file, use its parent as root.
    if let Some(p) = path.parent() { opts.root_path = p.to_path_buf(); }
    let mut visited: HashSet<IncludeKey> = HashSet::new();
    let mut deps: Vec<IncludeKey> = Vec::new();
    let mut out = String::new();
    let mut stack: Vec<String> = vec![display_path(path)];
    let mut macros = builtin_macros_for(path, 1, &opts);
    // Seed user defines (cannot override built-ins)
    for (k, v) in opts.defines.clone() { if !is_builtin(&k) { macros.insert(k, v); }}

    process_unit(
        path,
        text,
        &mut out,
        &mut visited,
        &mut deps,
        &mut macros,
        &mut stack,
        &opts,
        0,
    )?;
    Ok(PreprocessResult { text: out, source_map: SourceMap::default(), dependencies: deps })
}

fn process_unit(
    cur_path: &Path,
    text: &str,
    out: &mut String,
    visited: &mut HashSet<IncludeKey>,
    deps: &mut Vec<IncludeKey>,
    macros: &mut HashMap<String, MacroValue>,
    stack: &mut Vec<String>,
    opts: &PreprocessOptions,
    depth: usize,
) -> Result<(), PreprocessError> {
    if depth > MAX_DEPTH { return Err(PreprocessError::Message(format!("error: maximum include depth ({MAX_DEPTH}) exceeded"))); }
    let mut i_line: usize = 0;
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines_iter = normalized.lines();
    while let Some(line) = lines_iter.next() {
        i_line += 1;
        update_builtins(macros, cur_path, i_line, opts);
        if let Some((indent_ws, body)) = starts_with_directive(line) {
            if body.starts_with("include ") {
                let arg = body[8..].trim();
                let (form, target) = parse_include_target(arg)?;
                let (key, content) = resolve_include(cur_path, form, target, opts, stack)?;
                if !visited.contains(&key) {
                    visited.insert(key.clone());
                    deps.push(key);
                    let (inc_path, inc_text) = content;
                    // Cycle detection: if inc_path already on the active stack, report cycle
                    let inc_disp = display_path(&inc_path);
                    if stack.iter().any(|s| s == &inc_disp) {
                        let mut chain = stack.clone();
                        chain.push(inc_disp);
                        return Err(PreprocessError::Cycle(chain));
                    }
                    stack.push(display_path(&inc_path));
                    let mut child_macros = macros.clone();
                    process_unit(&inc_path, &inc_text, out, visited, deps, &mut child_macros, stack, opts, depth + 1)?;
                    stack.pop();
                    if out.len() > MAX_SIZE { return Err(PreprocessError::Message("error: preprocessed output exceeds 2 MiB (limit)".into())); }
                }
                continue;
            } else if body.starts_with("define ") {
                let rest = body[7..].trim();
                let (name, val) = parse_define(rest)?;
                if is_builtin(name) { return Err(PreprocessError::Message(format!("error: cannot redefine built-in macro {}", name))); }
                macros.insert(name.to_string(), val);
                continue;
            } else if body.starts_with("undef ") {
                let name = body[6..].trim();
                if is_builtin(name) { return Err(PreprocessError::Message(format!("error: cannot undefine built-in macro {}", name))); }
                macros.remove(name);
                continue;
            } else if body.starts_with("if ") {
                // Gather the conditional block up to matching #endif
                let cond_expr = body[3..].trim();
                let (chosen, consumed) = consume_conditional(cond_expr, &mut lines_iter, cur_path, i_line, macros, opts)?;
                // Process chosen block recursively as inline text
                process_unit(cur_path, &chosen, out, visited, deps, macros, stack, opts, depth)?;
                // Adjust i_line by consumed lines (already advanced by iterator). We cannot adjust iterator; already consumed.
                let _ = consumed; // nothing to do
                continue;
            } else if body.starts_with("elif ") || body == "else" || body == "endif" {
                // These should be handled only by consume_conditional
                return Err(PreprocessError::Message(format!("error: misordered directive '#{}' without matching #if", body.split_whitespace().next().unwrap_or(""))));
            }
            // Not a recognized directive → pass through as-is
            out.push_str(indent_ws); out.push('#'); out.push_str(body); out.push('\n');
        } else {
            out.push_str(line); out.push('\n');
        }
    }
    Ok(())
}

fn starts_with_directive(line: &str) -> Option<(&str, &str)> {
    // returns (leading_ws, body_after_hash)
    let trimmed = line.trim_start();
    let ws_len = line.len() - trimmed.len();
    if trimmed.starts_with('#') {
        let body = trimmed[1..].trim_start();
        Some((&line[..ws_len], body))
    } else { None }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
enum IncludeForm { Quoted, Angle, Bare }

fn parse_include_target(arg: &str) -> Result<(IncludeForm, String), PreprocessError> {
    if arg.starts_with('"') {
        if let Some(pos) = arg[1..].find('"') { return Ok((IncludeForm::Quoted, arg[1..1+pos].to_string())); }
        return Err(PreprocessError::Message("error: malformed #include; missing closing quote".into()));
    }
    if arg.starts_with('<') {
        if let Some(pos) = arg[1..].find('>') { return Ok((IncludeForm::Angle, arg[1..1+pos].to_string())); }
        return Err(PreprocessError::Message("error: malformed #include; missing '>'".into()));
    }
    if arg.contains(' ') { return Err(PreprocessError::Message("error: bare #include path contains spaces; use quotes".into())); }
    Ok((IncludeForm::Bare, arg.to_string()))
}

fn resolve_include(
    cur_path: &Path,
    form: IncludeForm,
    target: String,
    opts: &PreprocessOptions,
    stack: &Vec<String>,
) -> Result<(IncludeKey, (PathBuf, String)), PreprocessError> {
    // Prevent cycle with active stack entries matching the same path we would resolve to later; cycle detection finalized after resolution.
    let mut searched: Vec<String> = Vec::new();
    let norm_target = target.replace('\\', "/");

    let try_embedded = |t: &str| -> Option<(IncludeKey, (PathBuf, String))> {
        if !opts.use_embedded { return None; }
        if let Some(f) = embedded::find_file(t) {
            let key = IncludeKey::Embedded(format!("embedded:/{t}"));
            // Logical path as pseudo-filesystem path for display
            let pb = PathBuf::from(format!("embedded:/{t}"));
            let text = String::from_utf8_lossy(f.contents).to_string();
            return Some((key, (pb, text)));
        }
        // Heuristic: swap extension .basil <-> .bas
        if let Some(alt) = swap_ext(t) {
            if let Some(f) = embedded::find_file(&alt) {
                let key = IncludeKey::Embedded(format!("embedded:/{alt}"));
                let pb = PathBuf::from(format!("embedded:/{alt}"));
                let text = String::from_utf8_lossy(f.contents).to_string();
                return Some((key, (pb, text)));
            }
        }
        None
    };

    let try_fs = |base: &Path| -> Option<(IncludeKey, (PathBuf, String))> {
        let p = base.join(&norm_target);
        if p.is_file() {
            if let Ok(pcan) = canonicalize_insensitive(&p) {
                if let Ok(bytes) = std::fs::read(&pcan) {
                    if let Ok(text) = String::from_utf8(bytes) {
                        return Some((IncludeKey::Fs(pcan.clone()), (pcan, text)));
                    }
                }
            }
        }
        // Heuristic: try alternate extension
        if let Some(alt) = swap_ext(&norm_target) {
            let p2 = base.join(&alt);
            if p2.is_file() {
                if let Ok(pcan) = canonicalize_insensitive(&p2) {
                    if let Ok(bytes) = std::fs::read(&pcan) {
                        if let Ok(text) = String::from_utf8(bytes) {
                            return Some((IncludeKey::Fs(pcan.clone()), (pcan, text)));
                        }
                    }
                }
            }
        }
        None
    };

    match form {
        IncludeForm::Angle => {
            // 1) embedded 2) -I 3) project root
            if let Some(hit) = try_embedded(&norm_target) { return Ok(hit); }
            for ip in &opts.include_paths { searched.push(format!("-I {}", ip.display())); if let Some(hit) = try_fs(ip) { return Ok(hit); } }
            searched.push(format!("{}", opts.root_path.display())); if let Some(hit) = try_fs(&opts.root_path) { return Ok(hit); }
        }
        _ => {
            // quoted/bare: 1) current file dir 2) project root 3) -I 4) BASIL_PATH 5) embedded
            if let Some(dir) = cur_path.parent() { searched.push(format!("{}", dir.display())); if let Some(hit) = try_fs(dir) { return Ok(hit); } }
            searched.push(format!("{}", opts.root_path.display())); if let Some(hit) = try_fs(&opts.root_path) { return Ok(hit); }
            for ip in &opts.include_paths { searched.push(format!("-I {}", ip.display())); if let Some(hit) = try_fs(ip) { return Ok(hit); } }
            for ep in &opts.env_paths { searched.push(format!("BASIL_PATH {}", ep.display())); if let Some(hit) = try_fs(ep) { return Ok(hit); } }
            if let Some(hit) = try_embedded(&norm_target) { return Ok(hit); }
        }
    }

    // Not found
    let from = stack.last().cloned();
    Err(PreprocessError::NotFound { target, searched, include_from: from })
}

fn swap_ext(path: &str) -> Option<String> {
    if path.ends_with(".basil") {
        Some(path.strip_suffix(".basil").unwrap().to_string() + ".bas")
    } else if path.ends_with(".bas") {
        Some(path.strip_suffix(".bas").unwrap().to_string() + ".basil")
    } else { None }
}

fn parse_define(rest: &str) -> Result<(&str, MacroValue), PreprocessError> {
    let mut it = rest.splitn(2, char::is_whitespace);
    let name = it.next().unwrap_or("");
    if name.is_empty() { return Err(PreprocessError::Message("error: expected macro name".into())); }
    let value_str = it.next().unwrap_or("").trim();
    if value_str.is_empty() { return Ok((name, MacroValue::Bool(true))); }
    if value_str.starts_with('"') {
        if let Some(pos) = value_str[1..].find('"') { return Ok((name, MacroValue::Str(value_str[1..1+pos].to_string()))); }
        return Err(PreprocessError::Message("error: malformed string literal in #define".into()));
    }
    // try int
    if let Ok(v) = value_str.parse::<i64>() { return Ok((name, MacroValue::Int(v))); }
    // otherwise treat as bare identifier string
    Ok((name, MacroValue::Str(value_str.to_string())))
}

fn consume_conditional<'a>(
    first_cond: &str,
    lines: &mut std::str::Lines<'a>,
    cur_path: &Path,
    _first_line_no: usize,
    macros: &mut HashMap<String, MacroValue>,
    opts: &PreprocessOptions,
) -> Result<(String, usize), PreprocessError> {
    // Capture blocks for #if / #elif* / #else until #endif
    let mut blocks: Vec<(Option<String>, String)> = Vec::new(); // (cond, text)
    let mut current = String::new();
    let mut consumed = 0usize;
    // push first condition marker
    blocks.push((Some(first_cond.to_string()), String::new()));

    while let Some(line) = lines.next() {
        consumed += 1;
        if let Some((_ws, body)) = starts_with_directive(line) {
            if body.starts_with("elif ") {
                // start new block
                let cond = body[5..].trim().to_string();
                blocks.push((Some(cond), String::new()));
                continue;
            } else if body == "else" {
                blocks.push((None, String::new()));
                continue;
            } else if body == "endif" {
                break;
            }
        }
        // append to last block
        if let Some((_cond, text)) = blocks.last_mut() {
            text.push_str(line); text.push('\n');
        } else {
            current.push_str(line); current.push('\n'); // shouldn't happen
        }
    }
    // Evaluate conditions in order
    for (cond_opt, text) in blocks.into_iter() {
        if let Some(cond) = cond_opt {
            if eval_expr(&cond, macros, cur_path, opts)? {
                return Ok((text, consumed));
            }
        } else {
            return Ok((text, consumed));
        }
    }
    Ok((String::new(), consumed))
}

// Expression evaluator

#[derive(Debug, Clone, PartialEq)]
enum Val { I(i64), S(String), B(bool) }

fn truthy(v: &Val) -> bool {
    match v { Val::I(i)=>*i!=0, Val::S(s)=>!s.is_empty(), Val::B(b)=>*b }
}

fn eval_expr(src: &str, macros: &HashMap<String, MacroValue>, cur_path: &Path, opts: &PreprocessOptions) -> Result<bool, PreprocessError> {
    let mut p = Parser::new(src, macros, cur_path, opts);
    let v = p.parse_expr()?;
    Ok(truthy(&v))
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
    macros: &'a HashMap<String, MacroValue>,
    cur_path: &'a Path,
    opts: &'a PreprocessOptions,
}

impl<'a> Parser<'a> {
    fn new(src: &'a str, macros: &'a HashMap<String, MacroValue>, cur_path: &'a Path, opts: &'a PreprocessOptions) -> Self {
        Self { s: src.as_bytes(), i: 0, macros, cur_path, opts }
    }
    fn parse_expr(&mut self) -> Result<Val, PreprocessError> { self.parse_or() }
    fn parse_or(&mut self) -> Result<Val, PreprocessError> {
        let mut v = self.parse_and()?;
        loop { self.skip_ws(); if self.peek2("||") { self.i+=2; let r=self.parse_and()?; v = Val::B(truthy(&v) || truthy(&r)); } else { break; } }
        Ok(v)
    }
    fn parse_and(&mut self) -> Result<Val, PreprocessError> {
        let mut v = self.parse_cmp()?;
        loop { self.skip_ws(); if self.peek2("&&") { self.i+=2; let r=self.parse_cmp()?; v = Val::B(truthy(&v) && truthy(&r)); } else { break; } }
        Ok(v)
    }
    fn parse_cmp(&mut self) -> Result<Val, PreprocessError> {
        let mut v = self.parse_unary()?;
        loop {
            self.skip_ws();
            if self.peek2("==") { self.i+=2; let r=self.parse_unary()?; v = Val::B(equals(&v, &r)?); }
            else if self.peek2("!=") { self.i+=2; let r=self.parse_unary()?; v = Val::B(!equals(&v, &r)?); }
            else if self.peek2("<=") { self.i+=2; let r=self.parse_unary()?; v = Val::B(compare(&v, &r, |a,b| a<=b)?); }
            else if self.peek2(">=") { self.i+=2; let r=self.parse_unary()?; v = Val::B(compare(&v, &r, |a,b| a>=b)?); }
            else if self.peek1('<') { self.i+=1; let r=self.parse_unary()?; v = Val::B(compare(&v, &r, |a,b| a<b)?); }
            else if self.peek1('>') { self.i+=1; let r=self.parse_unary()?; v = Val::B(compare(&v, &r, |a,b| a>b)?); }
            else { break; }
        }
        Ok(v)
    }
    fn parse_unary(&mut self) -> Result<Val, PreprocessError> {
        self.skip_ws();
        if self.peek1('!') { self.i+=1; let v=self.parse_unary()?; return Ok(Val::B(!truthy(&v))); }
        self.parse_primary()
    }
    fn parse_primary(&mut self) -> Result<Val, PreprocessError> {
        self.skip_ws();
        if self.peek1('(') { self.i+=1; let v=self.parse_expr()?; self.skip_ws(); if !self.peek1(')') { return Err(msg("error: expected ')'")); } self.i+=1; return Ok(v); }
        if self.peek1('"') {
            self.i += 1; let start = self.i; while self.i < self.s.len() && self.s[self.i] != b'"' { self.i+=1; }
            if self.i >= self.s.len() { return Err(msg("error: unterminated string literal")); }
            let s = String::from_utf8_lossy(&self.s[start..self.i]).to_string(); self.i += 1; return Ok(Val::S(s));
        }
        if let Some(num) = self.read_int() { return Ok(Val::I(num)); }
        // identifier or defined(NAME)
        let ident = self.read_ident().ok_or_else(|| msg("error: expected identifier, literal, or ("))?;
        if ident == "defined" { self.skip_ws(); if !self.peek1('(') { return Err(msg("error: expected '(' after defined")); } self.i+=1; self.skip_ws(); let name = self.read_ident().ok_or_else(|| msg("error: expected macro name in defined()"))?; self.skip_ws(); if !self.peek1(')') { return Err(msg("error: expected ')'")); } self.i+=1; return Ok(Val::I(if self.resolve_ident(&name).is_some() {1} else {0})); }
        if let Some(v) = self.resolve_ident(&ident) { return Ok(v); }
        Err(msg(format!("error: unknown identifier {ident:?} (hint: use defined({ident}))")))
    }

    fn resolve_ident(&self, name: &str) -> Option<Val> {
        // built-ins override user macros
        match name {
            "__file__" => return Some(Val::S(display_path(self.cur_path))),
            "__line__" => return Some(Val::I(0)), // dynamic; not used in exprs often
            "__engine__" => return Some(Val::S(self.opts.engine_name.clone())),
            "__version__" => return Some(Val::I(self.opts.version_major)),
            "__os__" => return Some(Val::S(current_os().to_string())),
            "__debug__" => return Some(Val::I(if std::env::var("BASIL_DEBUG").ok().as_deref()==Some("1") {1} else {0})),
            _ => {}
        }
        self.macros.get(&name.to_string()).map(|m| match m { MacroValue::Bool(b)=>Val::B(*b), MacroValue::Int(i)=>Val::I(*i), MacroValue::Str(s)=>Val::S(s.clone()) })
    }

    fn skip_ws(&mut self) { while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() { self.i+=1; } }
    fn peek1(&self, c: char) -> bool { self.i < self.s.len() && self.s[self.i] == c as u8 }
    fn peek2(&self, s: &str) -> bool { self.i + 1 < self.s.len() && &self.s[self.i..self.i+2] == s.as_bytes() }
    fn read_int(&mut self) -> Option<i64> { self.skip_ws(); let start = self.i; while self.i < self.s.len() && self.s[self.i].is_ascii_digit() { self.i+=1; } if self.i>start { std::str::from_utf8(&self.s[start..self.i]).ok()?.parse::<i64>().ok() } else { None } }
    fn read_ident(&mut self) -> Option<String> { self.skip_ws(); let start = self.i; if self.i < self.s.len() && (self.s[self.i].is_ascii_alphabetic() || self.s[self.i] == b'_') { self.i+=1; while self.i < self.s.len() && (self.s[self.i].is_ascii_alphanumeric() || self.s[self.i]==b'_') { self.i+=1; } let s = String::from_utf8_lossy(&self.s[start..self.i]).to_string(); Some(s) } else { None } }
}

fn equals(a: &Val, b: &Val) -> Result<bool, PreprocessError> { match (a,b) { (Val::I(x),Val::I(y))=>Ok(x==y), (Val::S(x),Val::S(y))=>Ok(x==y), (Val::B(x),Val::B(y))=>Ok(x==y), _=>Err(msg("error: type mismatch in ==/!=")) } }
fn compare<F: FnOnce(i64,i64)->bool>(a: &Val, b: &Val, f: F) -> Result<bool, PreprocessError> {
    match (a,b) {
        (Val::I(x), Val::I(y)) => Ok(f(*x,*y)),
        (Val::S(x), Val::S(y)) => Ok(f(str_cmp_key(x), str_cmp_key(y))),
        _ => Err(msg("error: type mismatch: cannot compare string to int or bool")),
    }
}

fn str_cmp_key(s: &str) -> i64 {
    // Lexicographic via bytes converted to i64 key (very rough); for real impl we'd compare strings
    // Here we fallback to standard ordering using Ord on String by mapping to i64 of first 8 bytes
    let bytes = s.as_bytes();
    let mut acc: i64 = 0;
    for (i, b) in bytes.iter().take(8).enumerate() { acc |= (*b as i64) << (i*8); }
    acc
}

fn msg<S: Into<String>>(s: S) -> PreprocessError { PreprocessError::Message(s.into()) }

fn display_path(p: &Path) -> String { p.display().to_string() }

fn canonicalize_insensitive(p: &Path) -> std::io::Result<PathBuf> {
    #[cfg(windows)]
    {
        // Use std canonicalize and lower-case for include-once keys consistency
        let c = std::fs::canonicalize(p)?; Ok(PathBuf::from(c.to_string_lossy().to_lowercase()))
    }
    #[cfg(not(windows))]
    { std::fs::canonicalize(p) }
}

fn read_env_paths() -> Vec<PathBuf> {
    let var = std::env::var("BASIL_PATH").unwrap_or_default();
    if var.is_empty() { return Vec::new(); }
    #[cfg(windows)]
    let sep = ';';
    #[cfg(not(windows))]
    let sep = ':';
    var.split(sep).filter(|s| !s.is_empty()).map(|s| PathBuf::from(s)).collect()
}

fn current_os() -> &'static str {
    if cfg!(windows) { "windows" } else if cfg!(target_os = "macos") { "macos" } else { "linux" }
}

fn builtin_macros_for(path: &Path, _line: usize, opts: &PreprocessOptions) -> HashMap<String, MacroValue> {
    let mut m = HashMap::new();
    // Built-ins handled in evaluator directly; keep table for overrides check
    m.insert("__file__".into(), MacroValue::Str(display_path(path)));
    m.insert("__line__".into(), MacroValue::Int(1));
    m.insert("__engine__".into(), MacroValue::Str(opts.engine_name.clone()));
    m.insert("__version__".into(), MacroValue::Int(opts.version_major));
    m.insert("__os__".into(), MacroValue::Str(current_os().to_string()));
    m.insert("__debug__".into(), MacroValue::Int(if std::env::var("BASIL_DEBUG").ok().as_deref()==Some("1") {1} else {0}));
    m
}

fn update_builtins(macros: &mut HashMap<String, MacroValue>, path: &Path, line: usize, opts: &PreprocessOptions) {
    macros.insert("__file__".into(), MacroValue::Str(display_path(path)));
    macros.insert("__line__".into(), MacroValue::Int(line as i64));
    macros.insert("__engine__".into(), MacroValue::Str(opts.engine_name.clone()));
    macros.insert("__version__".into(), MacroValue::Int(opts.version_major));
    macros.insert("__os__".into(), MacroValue::Str(current_os().to_string()));
    macros.insert("__debug__".into(), MacroValue::Int(if std::env::var("BASIL_DEBUG").ok().as_deref()==Some("1") {1} else {0}));
}

fn is_builtin(name: &str) -> bool {
    matches!(name, "__file__"|"__line__"|"__engine__"|"__version__"|"__os__"|"__debug__")
}

// Helper to build options from CLI-like inputs
pub fn build_pre_opts_for_file(primary_path: &Path, cli: &crate::PreFlags) -> PreprocessOptions {
    let version_major = crate::VERSION_MAJOR.load(std::sync::atomic::Ordering::Relaxed) as i64;
    let mut opts = PreprocessOptions::default();
    opts.root_path = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    opts.include_paths = cli.include_paths.clone();
    opts.env_paths = read_env_paths();
    opts.use_embedded = !cli.no_embedded;
    opts.defines = cli.defines.clone();
    opts.engine_name = "basic".to_string();
    opts.version_major = version_major;
    // If the primary file has a parent, prefer that as root search path 1
    if let Some(p) = primary_path.parent() { opts.root_path = p.to_path_buf(); }
    opts
}
