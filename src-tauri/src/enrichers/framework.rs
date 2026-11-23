use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default)]
pub struct EnrichmentResult {
    pub framework: Option<String>,
    pub runtime: Option<String>,
    pub project_name: Option<String>,
}

#[derive(Debug, Clone, Default)]
struct CachedManifest {
    framework: Option<String>,
    runtime: Option<String>,
    project_name: Option<String>,
}

#[derive(Debug, Default)]
pub struct FrameworkEnricher {
    manifest_cache: HashMap<PathBuf, CachedManifest>,
}

impl FrameworkEnricher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enrich(
        &mut self,
        process_name: &str,
        exe_path: Option<&str>,
        command_line: &str,
        cwd: Option<&str>,
    ) -> EnrichmentResult {
        let name_lower = process_name.trim().to_ascii_lowercase();
        let exe_lower = exe_path
            .map(|s| s.trim().to_ascii_lowercase())
            .unwrap_or_default();
        let cmd_lower = command_line.trim().to_ascii_lowercase();

        let (mut framework, mut runtime) = detect_from_process(&name_lower, &exe_lower, &cmd_lower);
        let mut project_name = None;

        if let Some(cwd_str) = cwd {
            let cwd_path = PathBuf::from(cwd_str);
            if !is_generic_root(&cwd_path) {
                let manifest = self.read_manifest_cached(&cwd_path);
                if framework.is_none() {
                    framework = manifest.framework.clone();
                }
                if runtime.is_none() {
                    runtime = manifest.runtime.clone();
                }
                project_name = manifest.project_name.clone().or_else(|| {
                    cwd_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .map(|s| s.to_string())
                });
            }
        }

        EnrichmentResult {
            framework,
            runtime,
            project_name,
        }
    }

    fn read_manifest_cached(&mut self, dir: &Path) -> CachedManifest {
        if let Some(cached) = self.manifest_cache.get(dir) {
            return cached.clone();
        }
        let parsed = inspect_directory_manifests(dir);
        if self.manifest_cache.len() > 512 {
            self.manifest_cache.clear();
        }
        self.manifest_cache.insert(dir.to_path_buf(), parsed.clone());
        parsed
    }
}

fn is_generic_root(path: &Path) -> bool {
    let s = path.to_string_lossy();
    matches!(
        s.as_ref(),
        "" | "/" | "/home" | "/root" | "/tmp" | "/var" | "/usr" | "/etc" | "C:\\"
    )
}

fn detect_from_process(
    name: &str,
    exe: &str,
    cmd: &str,
) -> (Option<String>, Option<String>) {
    // Data & infrastructure services first
    if contains_any(name, cmd, &["postgres", "postmaster"]) {
        return (Some("PostgreSQL".into()), Some("Database".into()));
    }
    if contains_any(name, cmd, &["mariadbd", "mariadb"]) {
        return (Some("MariaDB".into()), Some("Database".into()));
    }
    if contains_any(name, cmd, &["mysqld", "mysql"]) {
        return (Some("MySQL".into()), Some("Database".into()));
    }
    if contains_any(name, cmd, &["mongod", "mongos"]) {
        return (Some("MongoDB".into()), Some("Database".into()));
    }
    if contains_any(name, cmd, &["redis-server", "valkey-server"]) {
        return (Some("Redis".into()), Some("Cache / Store".into()));
    }
    if contains_any(name, cmd, &["elasticsearch"]) {
        return (Some("Elasticsearch".into()), Some("Search".into()));
    }
    if contains_any(name, cmd, &["rabbitmq", "beam.smp"]) && cmd.contains("rabbit") {
        return (Some("RabbitMQ".into()), Some("Broker".into()));
    }
    if name == "minio" || cmd.contains("minio server") {
        return (Some("MinIO".into()), Some("Object Storage".into()));
    }

    // JS / TS runtimes & frameworks
    let js_runtime = if name == "bun" || exe.ends_with("/bun") || cmd.contains("bun ") {
        Some("Bun".to_string())
    } else if name == "deno" || exe.ends_with("/deno") {
        Some("Deno".to_string())
    } else if matches!(name, "node" | "nodejs" | "next-server" | "vite" | "tsx" | "nodemon")
        || exe.ends_with("/node")
        || exe.ends_with("\\node.exe")
        || cmd.contains("node_modules")
    {
        Some("Node.js".to_string())
    } else {
        None
    };

    if cmd.contains("next-server") || cmd.contains("next dev") || cmd.contains("/.bin/next") || cmd.contains("/next/dist/") {
        return (Some("Next.js".into()), js_runtime.or_else(|| Some("Node.js".into())));
    }
    if cmd.contains("vite") || name == "vite" {
        return (Some("Vite".into()), js_runtime.or_else(|| Some("Node.js".into())));
    }
    if cmd.contains("nuxt") || cmd.contains("nuxi") {
        return (Some("Nuxt".into()), js_runtime.or_else(|| Some("Node.js".into())));
    }
    if cmd.contains("astro") {
        return (Some("Astro".into()), js_runtime.or_else(|| Some("Node.js".into())));
    }
    if cmd.contains("remix") {
        return (Some("Remix".into()), js_runtime.or_else(|| Some("Node.js".into())));
    }
    if cmd.contains("nestjs") || cmd.contains("@nestjs") || cmd.contains("nest start") {
        return (Some("NestJS".into()), js_runtime.or_else(|| Some("Node.js".into())));
    }
    if cmd.contains("react-scripts") {
        return (Some("React".into()), js_runtime.or_else(|| Some("Node.js".into())));
    }
    if cmd.contains("express") {
        return (Some("Express".into()), js_runtime.or_else(|| Some("Node.js".into())));
    }
    if js_runtime.is_some() {
        return (None, js_runtime);
    }

    // Python runtimes & frameworks
    let is_python = name.starts_with("python")
        || matches!(name, "uvicorn" | "gunicorn" | "gunicorn3" | "streamlit" | "jupyter-lab" | "jupyter-notebook")
        || cmd.contains("python");
    if is_python {
        if cmd.contains("fastapi") {
            return (Some("FastAPI".into()), Some("Python".into()));
        }
        if name == "uvicorn" || cmd.contains("uvicorn") {
            return (Some("Uvicorn".into()), Some("Python".into()));
        }
        if name.starts_with("gunicorn") || cmd.contains("gunicorn") {
            return (Some("Gunicorn".into()), Some("Python".into()));
        }
        if cmd.contains("manage.py") && cmd.contains("runserver") || cmd.contains("django") {
            return (Some("Django".into()), Some("Python".into()));
        }
        if cmd.contains("flask") {
            return (Some("Flask".into()), Some("Python".into()));
        }
        if name == "streamlit" || cmd.contains("streamlit") {
            return (Some("Streamlit".into()), Some("Python".into()));
        }
        if name.starts_with("jupyter") || cmd.contains("jupyter") {
            return (Some("Jupyter".into()), Some("Python".into()));
        }
        return (None, Some("Python".into()));
    }

    // Systems & backend languages
    if name == "cargo" || cmd.contains("target/debug/") || cmd.contains("target/release/") {
        return (None, Some("Rust".into()));
    }
    if name == "go" || name == "air" || cmd.contains("go run") {
        return (None, Some("Go".into()));
    }
    if matches!(name, "ruby" | "puma" | "rails") || cmd.contains("rails server") || cmd.contains("bin/rails") {
        let fw = if cmd.contains("rails") {
            Some("Rails".into())
        } else {
            None
        };
        return (fw, Some("Ruby".into()));
    }
    if name == "java" || exe.ends_with("/java") || exe.ends_with("\\java.exe") {
        let fw = if cmd.contains("spring") {
            Some("Spring Boot".into())
        } else {
            None
        };
        return (fw, Some("Java".into()));
    }
    if name == "dotnet" || exe.ends_with("/dotnet") {
        return (Some("ASP.NET".into()), Some(".NET".into()));
    }
    if name.starts_with("php") {
        let fw = if cmd.contains("artisan") {
            Some("Laravel".into())
        } else {
            None
        };
        return (fw, Some("PHP".into()));
    }

    (None, None)
}

fn contains_any(name: &str, cmd: &str, needles: &[&str]) -> bool {
    needles
        .iter()
        .any(|needle| name.contains(needle) || cmd.contains(needle))
}

fn inspect_directory_manifests(dir: &Path) -> CachedManifest {
    let mut info = CachedManifest::default();

    let pkg_path = dir.join("package.json");
    if let Ok(raw) = fs::read_to_string(&pkg_path) {
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(&raw) {
            if let Some(name) = json.get("name").and_then(|v| v.as_str()) {
                info.project_name = Some(name.to_string());
            }
            info.runtime = Some("Node.js".to_string());
            let deps_has = |pkg: &str| -> bool {
                json.get("dependencies").and_then(|d| d.get(pkg)).is_some()
                    || json
                        .get("devDependencies")
                        .and_then(|d| d.get(pkg))
                        .is_some()
            };
            info.framework = if deps_has("next") {
                Some("Next.js".into())
            } else if deps_has("nuxt") {
                Some("Nuxt".into())
            } else if deps_has("astro") {
                Some("Astro".into())
            } else if deps_has("@remix-run/react") {
                Some("Remix".into())
            } else if deps_has("@nestjs/core") {
                Some("NestJS".into())
            } else if deps_has("vite") {
                Some("Vite".into())
            } else if deps_has("react") {
                Some("React".into())
            } else if deps_has("express") {
                Some("Express".into())
            } else {
                None
            };
            return info;
        }
    }

    let pyproject_path = dir.join("pyproject.toml");
    if let Ok(raw) = fs::read_to_string(&pyproject_path) {
        let lower = raw.to_ascii_lowercase();
        info.runtime = Some("Python".to_string());
        info.project_name = extract_toml_name(&raw);
        info.framework = if lower.contains("fastapi") {
            Some("FastAPI".into())
        } else if lower.contains("django") {
            Some("Django".into())
        } else if lower.contains("flask") {
            Some("Flask".into())
        } else if lower.contains("streamlit") {
            Some("Streamlit".into())
        } else {
            None
        };
        return info;
    }

    let cargo_path = dir.join("Cargo.toml");
    if let Ok(raw) = fs::read_to_string(&cargo_path) {
        info.runtime = Some("Rust".to_string());
        info.project_name = extract_toml_name(&raw);
        let lower = raw.to_ascii_lowercase();
        if lower.contains("axum") {
            info.framework = Some("Axum".into());
        } else if lower.contains("actix-web") {
            info.framework = Some("Actix Web".into());
        } else if lower.contains("rocket") {
            info.framework = Some("Rocket".into());
        }
        return info;
    }

    let go_mod_path = dir.join("go.mod");
    if let Ok(raw) = fs::read_to_string(&go_mod_path) {
        info.runtime = Some("Go".to_string());
        if let Some(first_line) = raw.lines().find(|l| l.trim_start().starts_with("module ")) {
            let mod_name = first_line.trim().trim_start_matches("module ").trim();
            info.project_name = mod_name.rsplit('/').next().map(|s| s.to_string());
        }
        return info;
    }

    if dir.join("Gemfile").exists() {
        info.runtime = Some("Ruby".to_string());
    } else if dir.join("pom.xml").exists() || dir.join("build.gradle").exists() {
        info.runtime = Some("Java".to_string());
    } else if dir.join("composer.json").exists() {
        info.runtime = Some("PHP".to_string());
    }

    info
}

fn extract_toml_name(content: &str) -> Option<String> {
    for line in content.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("name") {
            let rest = rest.trim_start();
            if let Some(val) = rest.strip_prefix('=') {
                let cleaned = val.trim().trim_matches('"').trim_matches('\'');
                if !cleaned.is_empty() {
                    return Some(cleaned.to_string());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn detects_frameworks_from_command_line() {
        let mut enricher = FrameworkEnricher::new();
        let next = enricher.enrich(
            "node",
            Some("/usr/bin/node"),
            "node /app/node_modules/.bin/next dev -p 3000",
            None,
        );
        assert_eq!(next.framework.as_deref(), Some("Next.js"));
        assert_eq!(next.runtime.as_deref(), Some("Node.js"));

        let uvicorn = enricher.enrich(
            "python3",
            Some("/usr/bin/python3"),
            "python3 -m uvicorn main:app --reload --port 8000",
            None,
        );
        assert_eq!(uvicorn.framework.as_deref(), Some("Uvicorn"));
        assert_eq!(uvicorn.runtime.as_deref(), Some("Python"));

        let redis = enricher.enrich(
            "redis-server",
            Some("/usr/bin/redis-server"),
            "redis-server 127.0.0.1:6379",
            None,
        );
        assert_eq!(redis.framework.as_deref(), Some("Redis"));
    }

    #[test]
    fn detects_manifest_from_temp_directory() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let tmp_dir = std::env::temp_dir().join(format!("iport_test_{stamp}"));
        fs::create_dir_all(&tmp_dir).unwrap();

        fs::write(
            tmp_dir.join("package.json"),
            r#"{"name":"admin-console","dependencies":{"vite":"^6.0.0","react":"^19.0.0"}}"#,
        )
        .unwrap();

        let mut enricher = FrameworkEnricher::new();
        let res = enricher.enrich("node", Some("/usr/bin/node"), "node dev.js", tmp_dir.to_str());
        assert_eq!(res.project_name.as_deref(), Some("admin-console"));
        assert_eq!(res.framework.as_deref(), Some("Vite"));
        assert_eq!(res.runtime.as_deref(), Some("Node.js"));

        let _ = fs::remove_dir_all(&tmp_dir);
    }

    #[test]
    fn detects_pyproject_and_cargo_manifests() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let py_dir = std::env::temp_dir().join(format!("iport_py_{stamp}"));
        let rs_dir = std::env::temp_dir().join(format!("iport_rs_{stamp}"));
        fs::create_dir_all(&py_dir).unwrap();
        fs::create_dir_all(&rs_dir).unwrap();

        fs::write(
            py_dir.join("pyproject.toml"),
            "[project]\nname = \"billing-api\"\ndependencies = [\"fastapi>=0.115\"]\n",
        )
        .unwrap();
        fs::write(
            rs_dir.join("Cargo.toml"),
            "[package]\nname = \"gateway-svc\"\n[dependencies]\naxum = \"0.8\"\n",
        )
        .unwrap();

        let mut enricher = FrameworkEnricher::new();
        let py = enricher.enrich("python3", None, "python3 main.py", py_dir.to_str());
        assert_eq!(py.project_name.as_deref(), Some("billing-api"));
        assert_eq!(py.framework.as_deref(), Some("FastAPI"));

        let rs = enricher.enrich("gateway-svc", None, "./gateway-svc", rs_dir.to_str());
        assert_eq!(rs.project_name.as_deref(), Some("gateway-svc"));
        assert_eq!(rs.framework.as_deref(), Some("Axum"));
        assert_eq!(rs.runtime.as_deref(), Some("Rust"));

        let _ = fs::remove_dir_all(&py_dir);
        let _ = fs::remove_dir_all(&rs_dir);
    }
}
