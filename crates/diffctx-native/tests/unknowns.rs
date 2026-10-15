// What the answer says it could not resolve, and what it can rule out
// (#505): an unknown is named, never read as nothing.

use std::path::Path;
use std::process::Command;

use tempfile::TempDir;

static BIN: std::sync::LazyLock<String> = std::sync::LazyLock::new(|| {
    std::env::var("DIFFCTX_NATIVE_BIN")
        .unwrap_or_else(|_| env!("CARGO_BIN_EXE_diffctx").to_string())
});

fn git(repo: &Path, args: &[&str]) {
    let status = Command::new("git")
        .current_dir(repo)
        .env_remove("GIT_DIR")
        .env_remove("GIT_WORK_TREE")
        .env_remove("GIT_INDEX_FILE")
        .args(args)
        .status()
        .expect("run git");
    assert!(status.success(), "git {args:?} failed");
}

/// A repository whose last commit applies `change` on top of `base`.
fn repo(base: &[(&str, &str)], change: &[(&str, &str)]) -> TempDir {
    let tmp = TempDir::new().expect("tempdir");
    fill(tmp.path(), base, change);
    tmp
}

fn fill(root: &Path, base: &[(&str, &str)], change: &[(&str, &str)]) {
    git(root, &["init", "-q", "."]);
    git(root, &["config", "user.email", "test@example.com"]);
    git(root, &["config", "user.name", "diffctx tests"]);
    for (step, files) in [base, change]
        .into_iter()
        .filter(|f| !f.is_empty())
        .enumerate()
    {
        for (rel, content) in files {
            let path = root.join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, content).unwrap();
        }
        git(root, &["add", "-A"]);
        git(root, &["commit", "-q", "-m", &format!("step {step}")]);
    }
}

fn symbol_markdown(repo: &Path, name: &str) -> String {
    let out = Command::new(&*BIN)
        .current_dir(repo)
        .args([".", "--symbol", name, "-q", "-f", "md"])
        .output()
        .expect("run diffctx");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8")
}

fn impact_markdown_within(repo: &Path, budget: &str) -> String {
    let out = Command::new(&*BIN)
        .current_dir(repo)
        .args([
            "--diff",
            "HEAD~1..HEAD",
            "--mode",
            "impact",
            "-q",
            "-f",
            "md",
            "--budget",
            budget,
        ])
        .output()
        .expect("run diffctx");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8")
}

fn impact_markdown(repo: &Path) -> String {
    let out = Command::new(&*BIN)
        .current_dir(repo)
        .args([
            "--diff",
            "HEAD~1..HEAD",
            "--mode",
            "impact",
            "-q",
            "-f",
            "md",
        ])
        .output()
        .expect("run diffctx");
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8(out.stdout).expect("utf8")
}

/// #389: `out = []; out.append(x)` appends to a list, whatever else defines
/// `append`. The same name through a `Report` is the caller, and through a
/// parameter nothing types it stays possible.
#[test]
fn a_builtin_container_receiver_is_no_caller_of_a_same_named_method() {
    let report = |body: &str| {
        format!(
            "class Report:\n    def __init__(self):\n        self.lines = []\n\n    def append(self, text):\n        {body}\n"
        )
    };
    let base = report("self.lines.append(text)");
    let tmp = repo(
        &[
            ("report.py", &base),
            (
                "lists.py",
                "def collect(xs):\n    out = []\n    for x in xs:\n        out.append(x)\n    return out\n",
            ),
            (
                "built.py",
                "def gather():\n    acc = list()\n    acc.append(1)\n    return acc\n",
            ),
            (
                "writer.py",
                "from report import Report\n\n\ndef write():\n    r = Report()\n    r.append(\"x\")\n",
            ),
            ("loose.py", "def emit(w):\n    w.append(\"y\")\n"),
        ],
        &[("report.py", &report("self.lines.append(text.strip())"))],
    );
    let md = impact_markdown(tmp.path());
    assert!(md.contains("called from writer.py::write"), "{md}");
    assert!(md.contains("possibly called from loose.py::emit"), "{md}");
    assert!(!md.contains("lists.py"), "{md}");
    assert!(!md.contains("built.py"), "{md}");
}

/// #389 on TypeScript: `new Map()` is a `Map`, and its `get` is not
/// `PreferencesService.get`.
#[test]
fn a_ts_builtin_map_receiver_is_no_caller_of_get() {
    let service = |n: u32| {
        format!(
            "export class PreferencesService {{\n  get(key: string): number {{\n    return key.length + {n};\n  }}\n}}\n"
        )
    };
    let tmp = repo(
        &[
            ("prefs.ts", &service(1)),
            (
                "cache.ts",
                "const m = new Map<string, number>();\n\nexport function look(k: string) {\n  return m.get(k);\n}\n",
            ),
            (
                "use.ts",
                "import { PreferencesService } from './prefs';\n\nexport function read() {\n  const p = new PreferencesService();\n  return p.get('a');\n}\n",
            ),
        ],
        &[("prefs.ts", &service(2))],
    );
    let md = impact_markdown(tmp.path());
    assert!(md.contains("called from use.ts::read"), "{md}");
    assert!(!md.contains("cache.ts"), "{md}");
}

/// #399: `cart.total` through a parameter nothing types may read the
/// changed property or any other object's. Listing it was noise; dropping
/// it read as "nothing calls them". It is counted as not resolved.
#[test]
fn a_read_through_an_untyped_receiver_is_counted_not_dropped() {
    let cart =
        |n: u32| format!("class Cart:\n    @property\n    def total(self):\n        return {n}\n");
    let tmp = repo(
        &[
            ("cart.py", &cart(1)),
            ("app.py", "def checkout(cart):\n    return cart.total\n"),
        ],
        &[("cart.py", &cart(2))],
    );
    let md = impact_markdown(tmp.path());
    assert!(
        md.contains("callers not resolved: 1 read(s) of `.total`"),
        "{md}"
    );
    assert!(!md.contains("nothing calls them"), "{md}");
    assert!(!md.contains("called from app.py"), "{md}");
}

/// A read and a call through the same untyped receiver (`if engine.switch:
/// engine.switch()`) is a possible caller: the call says more than the read.
#[test]
fn a_call_beside_a_read_through_one_receiver_stays_a_possible_caller() {
    let engine = |n: u32| format!("class Engine:\n    def switch(self):\n        return {n}\n");
    let tmp = repo(
        &[
            ("engine.py", &engine(1)),
            (
                "client.py",
                "def run(engine):\n    if engine.switch:\n        return engine.switch()\n",
            ),
        ],
        &[("engine.py", &engine(2))],
    );
    let md = impact_markdown(tmp.path());
    assert!(md.contains("possibly called from client.py::run"), "{md}");
    assert!(!md.contains("read(s) of `.switch`"), "{md}");
}

/// #427: a CI script feeds Python to the interpreter through a heredoc and
/// that code calls the function. Nothing reads embedded code, so the script
/// is named as not analysed; a script that only echoes the name is not.
#[test]
fn a_script_whose_embedded_code_names_the_symbol_is_named() {
    let tmp = repo(
        &[
            ("src/__init__.py", ""),
            (
                "src/seed_demo_data.py",
                "def seed_demo_data(n):\n    return list(range(n))\n",
            ),
            (
                "scripts/seed.sh",
                "#!/bin/sh\npython - <<'EOF'\nfrom src.seed_demo_data import seed_demo_data\nseed_demo_data(3)\nEOF\n",
            ),
            (
                "scripts/note.sh",
                "#!/bin/sh\necho \"run seed_demo_data after the migration\"\n",
            ),
            (
                "scripts/fmt.sh",
                "#!/bin/sh\npython3 - <<'PY'\n# seed_demo_data runs after this\nprint(\"seed_demo_data\")\nPY\n",
            ),
            (
                "tests/test_seed.py",
                "from src.seed_demo_data import seed_demo_data\n\n\ndef test_seed():\n    assert seed_demo_data(2) == [0, 1]\n",
            ),
        ],
        &[],
    );
    let md = symbol_markdown(tmp.path(), "seed_demo_data");
    assert!(md.contains("its embedded code is not analysed"), "{md}");
    assert!(md.contains("scripts/seed.sh"), "{md}");
    assert!(!md.contains("note.sh"), "{md}");
    assert!(!md.contains("fmt.sh"), "{md}");
    assert!(md.contains("called from tests/test_seed.py"), "{md}");
}

/// #393: the server config's only consumers name it by path, outside
/// Python: the entrypoint's command line and the Dockerfile that copies
/// it. A document or a comment that mentions it is no consumer.
#[test]
fn a_config_named_by_a_command_line_lists_where() {
    let conf = |n: u32| format!("workers = {n}\naccesslog = '-'\n");
    let tmp = repo(
        &[
            ("app/gunicorn_conf.py", &conf(2)),
            (
                "app/docker-entrypoint.sh",
                "#!/bin/sh\nexec gunicorn -c /app/gunicorn_conf.py \"src.app:server\"\n",
            ),
            (
                "app/Dockerfile",
                "FROM python:3.12\nCOPY gunicorn_conf.py docker-entrypoint.sh /app/\n",
            ),
            ("README.md", "Tune workers in app/gunicorn_conf.py.\n"),
            (
                "app/notes.sh",
                "#!/bin/sh\n# app/gunicorn_conf.py is read at boot\necho ok\n",
            ),
        ],
        &[("app/gunicorn_conf.py", &conf(4))],
    );
    let md = impact_markdown(tmp.path());
    assert!(
        md.contains("app/docker-entrypoint.sh:2 names app/gunicorn_conf.py"),
        "{md}"
    );
    assert!(
        md.contains("app/Dockerfile:2 names app/gunicorn_conf.py"),
        "{md}"
    );
    assert!(!md.contains("README.md"), "{md}");
    assert!(!md.contains("notes.sh"), "{md}");
    assert!(!md.contains("No resolved static callers"), "{md}");
}

/// #523: the settings and scripts that run a changed shell script by path,
/// absolute inside the repository or relative. A bare `lint.sh` names it
/// only while one file has that name.
#[test]
fn a_script_run_by_config_lists_the_entries_that_run_it() {
    let tmp = TempDir::new().expect("tempdir");
    let root = std::fs::canonicalize(tmp.path()).unwrap();
    let settings = format!(
        "{{\"hooks\": {{\"PreToolUse\": [\n  {{\"command\": \"{}/bin/guard.sh\"}},\n  {{\"command\": \"bin/lint.sh\"}}\n]}}}}\n",
        root.display()
    );
    fill(
        &root,
        &[
            (
                "bin/guard.sh",
                "#!/bin/bash\ncheck() { grep -q x <<<\"$1\"; }\ncheck \"$@\"\n",
            ),
            (
                "bin/lint.sh",
                "#!/bin/bash\nlint() { echo \"$1\"; }\nlint \"$@\"\n",
            ),
            ("other/lint.sh", "#!/bin/bash\necho other\n"),
            ("settings.json", &settings),
            (
                "package.json",
                "{\"scripts\": {\"guard\": \"./bin/guard.sh\"}}\n",
            ),
            ("tools/run.sh", "#!/bin/bash\nbash lint.sh\n"),
            (
                ".github/workflows/ci.yml",
                "on: push\njobs:\n  l:\n    runs-on: ubuntu-latest\n    steps:\n      - run: bash bin/lint.sh a\n",
            ),
        ],
        &[
            (
                "bin/guard.sh",
                "#!/bin/bash\ncheck() { grep -qi x <<<\"$1\"; }\ncheck \"$@\"\n",
            ),
            (
                "bin/lint.sh",
                "#!/bin/bash\nlint() { echo \"lint: $1\"; }\nlint \"$@\"\n",
            ),
        ],
    );
    let md = impact_markdown(&root);
    assert!(md.contains("settings.json:2 names bin/guard.sh"), "{md}");
    assert!(md.contains("settings.json:3 names bin/lint.sh"), "{md}");
    assert!(md.contains("package.json:1 names bin/guard.sh"), "{md}");
    assert!(
        md.contains(".github/workflows/ci.yml:6 names bin/lint.sh"),
        "{md}"
    );
    assert!(!md.contains("tools/run.sh"), "{md}");
}

/// #430: eight changed Rust methods named `new` and `build`, each defined
/// in four files, say one thing between them. They take one folded line,
/// and the function whose callers resolve keeps them.
#[test]
fn names_too_common_to_trace_fold_into_one_line() {
    let ty = |name: &str, n: u32| {
        format!(
            "pub struct {name}(pub u32);\nimpl {name} {{\n    pub fn new() -> Self {{ {name}({n}) }}\n    pub fn build(&self) -> u32 {{ self.0 + {n} }}\n}}\n"
        )
    };
    let lib = "pub mod a;\npub mod b;\npub mod c;\npub mod d;\n\npub fn entry() -> u32 {\n    a::A::new().0 + b::B::new().0 + c::C::new().0 + d::D::new().0 + a::unique_helper()\n}\n";
    let a = |n: u32| format!("{}pub fn unique_helper() -> u32 {{ {n} }}\n", ty("A", n));
    let tmp = repo(
        &[
            (
                "Cargo.toml",
                "[package]\nname = \"c\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            ("src/lib.rs", lib),
            ("src/a.rs", &a(1)),
            ("src/b.rs", &ty("B", 2)),
            ("src/c.rs", &ty("C", 3)),
            ("src/d.rs", &ty("D", 4)),
            (
                "src/main.rs",
                "fn main() {\n    println!(\"{}\", c::entry() + c::a::unique_helper());\n}\n",
            ),
        ],
        &[
            ("src/a.rs", &a(10)),
            ("src/b.rs", &ty("B", 20)),
            ("src/c.rs", &ty("C", 30)),
            ("src/d.rs", &ty("D", 40)),
        ],
    );
    let md = impact_markdown(tmp.path());
    assert!(
        md.contains("- 8 changed definition(s) with names too common to trace by name (rust has no binding model here"),
        "{md}"
    );
    assert!(!md.contains("callers not resolved: `new`"), "{md}");
    assert!(md.contains("called from src/main.rs::main"), "{md}");
    assert!(md.contains("called from src/lib.rs::entry"), "{md}");
}

/// What names a changed file by path is context, never at a caller's
/// expense: under a tight cap the same callers show with or without the
/// configs that name the file, its omitted-count line included.
#[test]
fn named_by_never_displaces_a_caller() {
    let lib = |n: u32| format!("def total(items):\n    return sum(items) * {n}\n");
    let callers: Vec<(String, String)> = (1..=6)
        .map(|i| {
            (
                format!("use{i}.py"),
                format!("from lib import total\n\n\ndef use{i}(x):\n    return total(x) + {i}\n"),
            )
        })
        .collect();
    let configs: Vec<(String, String)> = (1..=10)
        .map(|i| {
            (
                format!("jobs/run{i}.yml"),
                format!("steps:\n  - run: python lib.py --part {i}\n"),
            )
        })
        .collect();
    let base = |named: bool| {
        let mut files: Vec<(&str, &str)> = vec![("lib.py", "")];
        files.extend(callers.iter().map(|(p, c)| (p.as_str(), c.as_str())));
        if named {
            files.extend(configs.iter().map(|(p, c)| (p.as_str(), c.as_str())));
        }
        files
    };
    let (one, two) = (lib(1), lib(2));
    let build = |named: bool| {
        let mut files = base(named);
        files[0].1 = &one;
        repo(&files, &[("lib.py", &two)])
    };
    let (plain, named) = (build(false), build(true));
    for budget in ["120", "160", "200", "260", "400"] {
        let count = |md: &str| md.matches("called from use").count();
        let (a, b) = (
            impact_markdown_within(plain.path(), budget),
            impact_markdown_within(named.path(), budget),
        );
        assert_eq!(count(&a), count(&b), "budget {budget}:\n{a}\n---\n{b}");
    }
    let md = impact_markdown(named.path());
    assert!(md.contains("jobs/run1.yml:2 names lib.py"), "{md}");
}
