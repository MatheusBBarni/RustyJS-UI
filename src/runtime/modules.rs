use anyhow::{anyhow, Context as AnyhowContext, Result};
use boa_engine::{
    module::{Module, ModuleLoader, Referrer},
    Context as BoaContext, JsNativeError, JsResult, JsString, Source,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Default)]
pub(crate) struct AppModuleLoader {
    app_root: RefCell<Option<PathBuf>>,
    modules_by_path: RefCell<HashMap<PathBuf, Module>>,
    module_paths: RefCell<HashMap<Module, PathBuf>>,
    package_modules: RefCell<HashMap<String, Module>>,
}

impl AppModuleLoader {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn prepare_entry_module(
        &self,
        entry_path: &Path,
        context: &mut BoaContext<'_>,
    ) -> Result<(PathBuf, Module)> {
        let canonical_entry = entry_path
            .canonicalize()
            .with_context(|| format!("failed to read app entry `{}`", entry_path.display()))?;
        let app_root = canonical_entry
            .parent()
            .ok_or_else(|| {
                anyhow!(
                    "failed to determine app root for `{}`",
                    canonical_entry.display()
                )
            })?
            .to_path_buf();

        self.reset(app_root);

        let source = Source::from_filepath(&canonical_entry).map_err(|error| {
            anyhow!(
                "failed to open JS module `{}`: {error}",
                canonical_entry.display()
            )
        })?;
        let module = Module::parse(source, None, context).map_err(|error| {
            anyhow!(
                "failed to parse JS module `{}`: {error}",
                canonical_entry.display()
            )
        })?;

        self.cache_module(canonical_entry.clone(), module.clone());

        Ok((canonical_entry, module))
    }

    fn reset(&self, app_root: PathBuf) {
        *self.app_root.borrow_mut() = Some(app_root);
        self.modules_by_path.borrow_mut().clear();
        self.module_paths.borrow_mut().clear();
        self.package_modules.borrow_mut().clear();
    }

    fn cache_module(&self, path: PathBuf, module: Module) {
        self.modules_by_path
            .borrow_mut()
            .insert(path.clone(), module.clone());
        self.module_paths.borrow_mut().insert(module, path);
    }

    fn load_module(
        &self,
        referrer: &Referrer,
        specifier: JsString,
        context: &mut BoaContext<'_>,
    ) -> JsResult<Module> {
        let specifier = specifier
            .to_std_string()
            .map_err(|error| JsNativeError::typ().with_message(error.to_string()))?;
        if let Some(module) = self.load_builtin_package_module(&specifier, context)? {
            return Ok(module);
        }

        let (importer_path, importer_label) = self.resolve_importer(referrer)?;
        let resolved_path =
            self.resolve_import_path(importer_path.as_deref(), &importer_label, &specifier)?;

        if let Some(module) = self.modules_by_path.borrow().get(&resolved_path).cloned() {
            return Ok(module);
        }

        let source = Source::from_filepath(&resolved_path).map_err(|error| {
            JsNativeError::typ().with_message(format!(
                "failed to open import `{specifier}` from `{importer_label}`: {error}"
            ))
        })?;
        let module = Module::parse(source, None, context).map_err(|error| {
            JsNativeError::syntax().with_message(format!(
                "failed to parse import `{specifier}` from `{importer_label}`: {error}"
            ))
        })?;

        self.cache_module(resolved_path, module.clone());

        Ok(module)
    }

    fn load_builtin_package_module(
        &self,
        specifier: &str,
        context: &mut BoaContext<'_>,
    ) -> JsResult<Option<Module>> {
        if specifier != "RustyJS-UI" {
            return Ok(None);
        }

        if let Some(module) = self.package_modules.borrow().get(specifier).cloned() {
            return Ok(Some(module));
        }

        let module = Module::parse(Source::from_bytes(BUILTIN_RUSTYJS_UI_MODULE), None, context)
            .map_err(|error| {
                JsNativeError::syntax().with_message(format!(
                    "failed to parse built-in package `{specifier}`: {error}"
                ))
            })?;

        self.package_modules
            .borrow_mut()
            .insert(specifier.to_string(), module.clone());

        Ok(Some(module))
    }

    fn resolve_importer(&self, referrer: &Referrer) -> JsResult<(Option<PathBuf>, String)> {
        match referrer {
            Referrer::Module(module) => {
                let importer_path =
                    self.module_paths
                        .borrow()
                        .get(module)
                        .cloned()
                        .ok_or_else(|| {
                            JsNativeError::typ()
                                .with_message("missing source path for importing module")
                        })?;
                let importer_label = importer_path.display().to_string();
                Ok((Some(importer_path), importer_label))
            }
            Referrer::Realm(_) | Referrer::Script(_) => {
                let app_root = self.app_root()?;
                Ok((None, app_root.display().to_string()))
            }
        }
    }

    fn resolve_import_path(
        &self,
        importer_path: Option<&Path>,
        importer_label: &str,
        specifier: &str,
    ) -> JsResult<PathBuf> {
        if !(specifier.starts_with("./") || specifier.starts_with("../")) {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "unsupported import `{specifier}` from `{importer_label}`: only `./` and `../` specifiers are supported"
                ))
                .into());
        }

        if !specifier.ends_with(".js") {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "unsupported import `{specifier}` from `{importer_label}`: specifiers must include the `.js` extension"
                ))
                .into());
        }

        let app_root = self.app_root()?;
        let base_dir = importer_path
            .and_then(Path::parent)
            .map(Path::to_path_buf)
            .unwrap_or_else(|| app_root.clone());
        let resolved_path = base_dir.join(specifier).canonicalize().map_err(|error| {
            JsNativeError::typ().with_message(format!(
                "failed to resolve import `{specifier}` from `{importer_label}`: {error}"
            ))
        })?;

        if !resolved_path.starts_with(&app_root) {
            return Err(JsNativeError::typ()
                .with_message(format!(
                    "import `{specifier}` from `{importer_label}` resolves outside the app root `{}`",
                    app_root.display()
                ))
                .into());
        }

        Ok(resolved_path)
    }

    fn app_root(&self) -> JsResult<PathBuf> {
        self.app_root.borrow().clone().ok_or_else(|| {
            JsNativeError::typ()
                .with_message("module loader is not configured for an app entry")
                .into()
        })
    }
}

const BUILTIN_RUSTYJS_UI_MODULE: &str = r#"
const App = globalThis.App;
const View = globalThis.View;
const Text = globalThis.Text;
const Button = globalThis.Button;
const TextInput = globalThis.TextInput;
const SelectInput = globalThis.SelectInput;
const FlatList = globalThis.FlatList;
const Modal = globalThis.Modal;
const fetch = globalThis.fetch;

export { App, View, Text, Button, TextInput, SelectInput, FlatList, Modal, fetch };
"#;

impl ModuleLoader for AppModuleLoader {
    fn load_imported_module(
        &self,
        referrer: Referrer,
        specifier: JsString,
        finish_load: Box<dyn FnOnce(JsResult<Module>, &mut BoaContext<'_>)>,
        context: &mut BoaContext<'_>,
    ) {
        let result = self.load_module(&referrer, specifier, context);
        finish_load(result, context);
    }
}
