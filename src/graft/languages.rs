use anyhow::{Result, anyhow};
use std::ffi::c_void;
use std::mem;
use tree_sitter::Language;

pub struct LanguageSpec {
    pub name: &'static str,
    pub extensions: &'static [&'static str],
    pub get_language: fn() -> Language,
}

// Category A: Compatible
fn lang_rust() -> Language {
    tree_sitter_rust::LANGUAGE.into()
}
fn lang_js() -> Language {
    tree_sitter_javascript::LANGUAGE.into()
}
fn lang_ts() -> Language {
    tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()
}
fn lang_tsx() -> Language {
    tree_sitter_typescript::LANGUAGE_TSX.into()
}
fn lang_go() -> Language {
    tree_sitter_go::LANGUAGE.into()
}
fn lang_py() -> Language {
    tree_sitter_python::LANGUAGE.into()
}
fn lang_c() -> Language {
    tree_sitter_c::LANGUAGE.into()
}
fn lang_cpp() -> Language {
    tree_sitter_cpp::LANGUAGE.into()
}
fn lang_java() -> Language {
    tree_sitter_java::LANGUAGE.into()
}
fn lang_json() -> Language {
    tree_sitter_json::LANGUAGE.into()
}
fn lang_html() -> Language {
    tree_sitter_html::LANGUAGE.into()
}
fn lang_css() -> Language {
    tree_sitter_css::LANGUAGE.into()
}
fn lang_bash() -> Language {
    tree_sitter_bash::LANGUAGE.into()
}
fn lang_ocaml() -> Language {
    tree_sitter_ocaml::LANGUAGE_OCAML.into()
}
fn lang_ocaml_interface() -> Language {
    tree_sitter_ocaml::LANGUAGE_OCAML_INTERFACE.into()
}

// Category B: Language wrapper mismatch
// We expect `old` to be a struct wrapping a pointer.
fn transmute_lang<L>(old: L) -> Language {
    unsafe { mem::transmute_copy(&old) }
}

fn lang_markdown() -> Language {
    transmute_lang(tree_sitter_markdown::language())
}
fn lang_toml() -> Language {
    transmute_lang(tree_sitter_toml::language())
}
fn lang_dockerfile() -> Language {
    transmute_lang(tree_sitter_dockerfile::language())
}

// Category C: LanguageFn mismatch
// LanguageFn is repr(transparent) wrapper around unsafe extern "C" fn() -> *const TSLanguage
fn extract_from_fn<F>(f: F) -> Language {
    // Transmute the struct F (LanguageFn) into a function pointer
    let fn_ptr: unsafe extern "C" fn() -> *const c_void = unsafe { mem::transmute_copy(&f) };
    let ptr = unsafe { fn_ptr() };
    // Transmute the raw pointer to Language
    unsafe { mem::transmute(ptr) }
}

fn lang_lua() -> Language {
    extract_from_fn(tree_sitter_lua::LANGUAGE)
}
fn lang_nix() -> Language {
    extract_from_fn(tree_sitter_nix::LANGUAGE)
}
fn lang_yaml() -> Language {
    extract_from_fn(tree_sitter_yaml::LANGUAGE)
}
fn lang_make() -> Language {
    extract_from_fn(tree_sitter_make::LANGUAGE)
}

pub const LANGUAGES: &[LanguageSpec] = &[
    LanguageSpec {
        name: "Rust",
        extensions: &["rust", "rs"],
        get_language: lang_rust,
    },
    LanguageSpec {
        name: "JavaScript",
        extensions: &["javascript", "js", "jsx"],
        get_language: lang_js,
    },
    LanguageSpec {
        name: "TypeScript",
        extensions: &["typescript", "ts"],
        get_language: lang_ts,
    },
    LanguageSpec {
        name: "TSX",
        extensions: &["tsx"],
        get_language: lang_tsx,
    },
    LanguageSpec {
        name: "Go",
        extensions: &["go"],
        get_language: lang_go,
    },
    LanguageSpec {
        name: "Python",
        extensions: &["python", "py"],
        get_language: lang_py,
    },
    LanguageSpec {
        name: "C",
        extensions: &["c", "h"],
        get_language: lang_c,
    },
    LanguageSpec {
        name: "C++",
        extensions: &["cpp", "c++", "cc", "cxx", "hpp", "hxx"],
        get_language: lang_cpp,
    },
    LanguageSpec {
        name: "Java",
        extensions: &["java"],
        get_language: lang_java,
    },
    LanguageSpec {
        name: "JSON",
        extensions: &["json"],
        get_language: lang_json,
    },
    LanguageSpec {
        name: "HTML",
        extensions: &["html"],
        get_language: lang_html,
    },
    LanguageSpec {
        name: "CSS",
        extensions: &["css"],
        get_language: lang_css,
    },
    LanguageSpec {
        name: "Bash",
        extensions: &["bash", "sh", "zsh"],
        get_language: lang_bash,
    },
    LanguageSpec {
        name: "OCaml",
        extensions: &["ocaml", "ml"],
        get_language: lang_ocaml,
    },
    LanguageSpec {
        name: "OCaml Interface",
        extensions: &["ocaml_interface", "mli"],
        get_language: lang_ocaml_interface,
    },
    LanguageSpec {
        name: "Lua",
        extensions: &["lua"],
        get_language: lang_lua,
    },
    LanguageSpec {
        name: "Nix",
        extensions: &["nix"],
        get_language: lang_nix,
    },
    LanguageSpec {
        name: "YAML",
        extensions: &["yaml", "yml"],
        get_language: lang_yaml,
    },
    LanguageSpec {
        name: "Markdown",
        extensions: &["markdown", "md"],
        get_language: lang_markdown,
    },
    LanguageSpec {
        name: "TOML",
        extensions: &["toml"],
        get_language: lang_toml,
    },
    LanguageSpec {
        name: "Dockerfile",
        extensions: &["dockerfile", "docker"],
        get_language: lang_dockerfile,
    },
    LanguageSpec {
        name: "Make",
        extensions: &["make", "makefile", "mk"],
        get_language: lang_make,
    },
];

pub fn get_language(lang_name: &str) -> Result<Language> {
    for lang in LANGUAGES {
        if lang.extensions.contains(&lang_name) || lang.name.eq_ignore_ascii_case(lang_name) {
            return Ok((lang.get_language)());
        }
    }
    Err(anyhow!("Unsupported language: {}", lang_name))
}
