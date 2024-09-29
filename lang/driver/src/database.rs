use crate::result::DriverError;
use crate::{cache::*, Error, FileSource};
use std::rc::Rc;
use std::sync::Arc;

use crate::dependency_graph::DependencyGraph;
use ast::Exp;
use ast::HashSet;
use elaborator::normalizer::normalize::Normalize;
use elaborator::{build_type_info_table, ModuleTypeInfoTable, TypeInfoTable};
use lowering::{ModuleSymbolTable, SymbolTable};
use parser::cst;
use parser::cst::decls::UseDecl;
use renaming::Rename;
use url::Url;

use crate::fs::*;
use crate::info::*;

use rust_lapper::Lapper;

/// A database tracking a set of source files
pub struct Database {
    /// The source provider of the files (file system or in-memory)
    pub source: Box<dyn FileSource>,
    /// Dependency graph for each module
    pub deps: DependencyGraph,
    /// The source code text of each file
    pub files: Cache<codespan::File<String>>,
    /// The CST of each file (once parsed)
    pub cst: Cache<Result<Arc<cst::decls::Module>, Error>>,
    /// The symbol table constructed during lowering
    pub symbol_table: Cache<Arc<lowering::ModuleSymbolTable>>,
    /// The lowered, but not yet typechecked, UST
    pub ust: Cache<Result<Arc<ast::Module>, Error>>,
    /// The typechecked AST of a module
    pub ast: Cache<Result<Arc<ast::Module>, Error>>,
    /// The type info table constructed during typechecking
    pub type_info_table: Cache<elaborator::ModuleTypeInfoTable>,
    /// Hover information for spans
    pub info_by_id: Cache<Lapper<u32, Info>>,
    /// Spans of top-level items
    pub item_by_id: Cache<Lapper<u32, Item>>,
}

impl Database {
    // Core API
    //
    // The core API of the Database consists of functions which have the following forms:
    //
    // ```text
    // pub fn xxx(&mut self, uri: Url) -> Result<xxx, Error>
    // fn recompute_xxx(&mut self, uri: Url) -> Result<(), Error>
    // ```
    // where `xxx` can be the cst, ust, ast, or any other sort of information about a module.
    // These functions are all implemented in a similar way.
    //
    // The function `xxx(&mut self, uri: Url)` checks whether the desired object is already in the
    // cache. If it is in the cache and isn't marked as stale we immediately return the object.
    // Otherwise we call `recompute_xxx` which contains the logic to compute the object anew
    // and put it back in the cache.
    //
    // The function `recompute_xxx(&mut self, uri: Url)` generally proceeds in the following way:
    //
    // 1. We look into the dependency graph to find out what the direct dependencies
    //    of the module are.
    // 2. For each of the direct dependencies we use the `xxx(...)` functions to obtain the
    //    information that is required to recompute the object. For example, we obtain the
    //    symbol tables for renaming or the lookup tables for typechecking a module.
    //    These calls can trigger further computations if the information is not in one of the
    //    caches.

    // Core API: Source
    //
    //

    pub fn source(&mut self, uri: &Url) -> Result<String, Error> {
        match self.files.get_unless_stale(uri) {
            Some(file) => Ok(file.source().to_string()),
            None => self.recompute_source(uri),
        }
    }

    fn recompute_source(&mut self, uri: &Url) -> Result<String, Error> {
        let source = self.source.read_to_string(uri)?;
        let file = codespan::File::new(uri.as_str().into(), source.clone());
        self.files.insert(uri.clone(), file);
        Ok(source)
    }

    // Core API: CST (Concrete Syntax Tree)
    //
    //

    pub fn cst(&mut self, uri: &Url) -> Result<Arc<cst::decls::Module>, Error> {
        match self.cst.get_unless_stale(uri) {
            Some(cst) => cst.clone(),
            None => self.recompute_cst(uri),
        }
    }

    fn recompute_cst(&mut self, uri: &Url) -> Result<Arc<cst::decls::Module>, Error> {
        let source = self.source(uri)?;
        log::debug!("Parsing module: {}", uri);
        let module =
            parser::parse_module(uri.clone(), &source).map_err(Error::Parser).map(Arc::new);
        self.cst.insert(uri.clone(), module.clone());
        module
    }

    // Core API: SymbolTable
    //
    //

    pub fn symbol_table(&mut self, uri: &Url) -> Result<Arc<ModuleSymbolTable>, Error> {
        match self.symbol_table.get_unless_stale(uri) {
            Some(symbol_table) => Ok(symbol_table.clone()),
            None => self.recompute_symbol_table(uri),
        }
    }

    fn recompute_symbol_table(&mut self, uri: &Url) -> Result<Arc<ModuleSymbolTable>, Error> {
        let cst = self.cst(uri)?;
        let module_symbol_table = lowering::build_symbol_table(&cst).map(Arc::new)?;
        self.symbol_table.insert(uri.clone(), module_symbol_table.clone());
        Ok(module_symbol_table)
    }

    // Core API: UST
    //
    //

    pub fn ust(&mut self, uri: &Url) -> Result<Arc<ast::Module>, Error> {
        match self.ust.get_unless_stale(uri) {
            Some(ust) => ust.clone(),
            None => self.recompute_ust(uri),
        }
    }

    pub fn recompute_ust(&mut self, uri: &Url) -> Result<Arc<ast::Module>, Error> {
        let cst = self.cst(uri)?;
        let deps = self
            .deps
            .get(uri)
            .ok_or(Error::Driver(DriverError::Impossible(format!("Did not find deps for {}", uri))))
            .cloned()?;

        // Compute the SymbolTable consisting of all the
        // ModuleSymbolTables of all direct dependencies
        // and the SymbolTable from the module itself.
        let mut symbol_table = SymbolTable::default();
        let module_symbol_table = self.symbol_table(uri)?;
        symbol_table.insert(uri.clone(), module_symbol_table);
        for dep in deps {
            let module_symbol_table = self.symbol_table(&dep)?;
            symbol_table.insert(dep.clone(), module_symbol_table);
        }

        let ust = lowering::lower_module_with_symbol_table(&cst, &symbol_table)
            .map_err(Error::Lowering)
            .map(Arc::new);

        self.ust.insert(uri.clone(), ust.clone());
        ust
    }

    // Core API: TypeInfoTable
    //
    //

    pub fn type_info_table(&mut self, uri: &Url) -> Result<ModuleTypeInfoTable, Error> {
        match self.type_info_table.get_unless_stale(uri) {
            Some(table) => Ok(table.clone()),
            None => self.recompute_type_info_table(uri),
        }
    }

    pub fn recompute_type_info_table(&mut self, uri: &Url) -> Result<ModuleTypeInfoTable, Error> {
        let ust = self.ust(uri)?;
        let info_table = build_type_info_table(&ust);
        self.type_info_table.insert(uri.clone(), info_table.clone());
        Ok(info_table)
    }

    // Core API: AST
    //
    //

    pub fn load_module(&mut self, uri: &Url) -> Result<Arc<ast::Module>, Error> {
        log::debug!("Loading module: {}", uri);
        self.source(uri)?;
        self.build_dependency_dag()?;

        log::trace!("");
        log::trace!("Dependency graph:");
        log::trace!("");
        self.deps.print_dependency_tree();
        log::trace!("");

        self.load_ast(uri)
    }

    pub fn load_ast(&mut self, uri: &Url) -> Result<Arc<ast::Module>, Error> {
        log::trace!("Loading AST: {}", uri);

        match self.ast.get_unless_stale(uri) {
            Some(ast) => ast.clone(),
            None => {
                log::trace!("AST is stale, reloading");
                let ust = self.ust(uri).map(|x| (*x).clone())?;

                // Compute the dependencies
                let empty_vec = Vec::new();
                let direct_dependencies = self.deps.get(uri).unwrap_or(&empty_vec).clone();

                // Compute the type info table
                let mut info_table = TypeInfoTable::default();
                let mod_info_table = self.type_info_table(uri)?;
                info_table.insert(uri.clone(), mod_info_table);
                for dep_url in direct_dependencies {
                    let mod_info_table = self.type_info_table(&dep_url)?;
                    info_table.insert(dep_url.clone(), mod_info_table);
                }
                let ast =
                    elaborator::typechecker::check_with_lookup_table(Rc::new(ust), &info_table)
                        .map(Arc::new)
                        .map_err(Error::Type);

                self.ast.insert(uri.clone(), ast.clone());
                if let Ok(module) = &ast {
                    let (info_lapper, item_lapper) = collect_info(module.clone());
                    self.info_by_id.insert(uri.clone(), info_lapper);
                    self.item_by_id.insert(uri.clone(), item_lapper);
                }
                ast
            }
        }
    }

    // Creation
    //
    // The following methods provide various means to construct a driver instance.

    /// Create a new database that only keeps files in memory
    pub fn in_memory() -> Self {
        Self::from_source(InMemorySource::new())
    }

    /// Create a new database with the given source
    pub fn from_source(source: impl FileSource + 'static) -> Self {
        Self {
            source: Box::new(source),
            files: Cache::default(),
            deps: DependencyGraph::default(),
            cst: Cache::default(),
            symbol_table: Cache::default(),
            ust: Cache::default(),
            ast: Cache::default(),
            type_info_table: Cache::default(),
            info_by_id: Cache::default(),
            item_by_id: Cache::default(),
        }
    }

    // Utility Functions
    //
    // The following utility functions do not belong to the core API described above.

    /// Get the source of the files
    pub fn file_source(&self) -> &dyn FileSource {
        &*self.source
    }

    /// Get a mutable reference to the source of the files
    pub fn file_source_mut(&mut self) -> &mut Box<dyn FileSource> {
        &mut self.source
    }

    /// Invalidate the file behind the given URI and all its reverse dependencies
    pub fn invalidate(&mut self, uri: &Url) -> Result<(), Error> {
        self.invalidate_impl(uri);
        self.build_dependency_dag()?;
        let rev_deps: HashSet<Url> =
            self.deps.reverse_dependencies(uri).into_iter().cloned().collect();
        log::debug!(
            "Invalidating {} and its reverse dependencies: {:?}",
            uri,
            rev_deps.iter().map(ToString::to_string).collect::<Vec<_>>()
        );
        for rev_dep in &rev_deps {
            self.invalidate_impl(rev_dep);
        }
        Ok(())
    }

    fn invalidate_impl(&mut self, uri: &Url) {
        self.files.invalidate(uri);
        self.cst.invalidate(uri);
        self.symbol_table.invalidate(uri);
        self.ust.invalidate(uri);
        self.ast.invalidate(uri);
        self.type_info_table.invalidate(uri);
        self.info_by_id.invalidate(uri);
        self.item_by_id.invalidate(uri);
    }

    pub fn run(&mut self, uri: &Url) -> Result<Option<Box<Exp>>, Error> {
        let ast = self.load_module(uri)?;

        let main = ast.find_main();

        match main {
            Some(exp) => {
                let nf = exp.normalize_in_empty_env(&ast)?;
                Ok(Some(nf))
            }
            None => Ok(None),
        }
    }

    pub fn pretty_error(&self, uri: &Url, err: Error) -> miette::Report {
        let miette_error: miette::Error = err.into();
        let source = &self.files.get_even_if_stale(uri).unwrap().source;
        miette_error.with_source_code(miette::NamedSource::new(uri, source.to_owned()))
    }

    pub fn write_source(&mut self, uri: &Url, source: &str) -> Result<(), Error> {
        self.invalidate(uri)?;
        self.source.write_string(uri, source).map_err(|err| err.into())
    }

    pub fn print_to_string(&mut self, uri: &Url) -> Result<String, Error> {
        let module = self.load_ast(uri)?;
        let module = (*module).clone().rename();
        Ok(printer::Print::print_to_string(&module, None))
    }

    pub fn load_imports(&mut self, module_uri: &Url) -> Result<(), Error> {
        self.build_dependency_dag()?;
        let empty_vec = Vec::new();
        let direct_deps = self.deps.get(module_uri).unwrap_or(&empty_vec).clone();
        for direct_dep in direct_deps {
            self.load_ast(&direct_dep)?;
        }
        Ok(())
    }

    /// Builds the dependency DAG for a given module and checks for cycles.
    ///
    /// Returns a `HashMap` where each key is a module `Url` and the corresponding value
    /// is a vector of `Url`s representing the modules it depends on.
    ///
    /// # Errors
    ///
    /// Returns an error if a cycle is detected or if a module cannot be found or loaded.
    pub fn build_dependency_dag(&mut self) -> Result<(), Error> {
        let mut visited = HashSet::default();
        let mut stack = Vec::new();
        let mut graph = DependencyGraph::default();
        let modules: Vec<Url> = self.files.keys().cloned().collect();
        for module_uri in modules {
            self.visit_module(&module_uri, &mut visited, &mut stack, &mut graph)?;
        }
        self.deps = graph;
        Ok(())
    }

    /// Recursively visits a module, adds its dependencies to the graph, and checks for cycles.
    fn visit_module(
        &mut self,
        module_uri: &Url,
        visited: &mut HashSet<Url>,
        stack: &mut Vec<Url>,
        graph: &mut DependencyGraph,
    ) -> Result<(), Error> {
        if stack.contains(module_uri) {
            // Cycle detected
            let cycle = stack.to_vec();
            return Err(DriverError::ImportCycle(module_uri.clone(), cycle).into());
        }

        if visited.contains(module_uri) {
            // Module already processed
            return Ok(());
        }

        visited.insert(module_uri.clone());
        stack.push(module_uri.clone());

        let module = self.cst(module_uri)?;

        // Collect dependencies from `use` declarations
        let mut dependencies = Vec::new();
        for use_decl in &module.use_decls {
            let UseDecl { path, .. } = use_decl;
            // Resolve the module name to a `Url`
            let dep_url = self.resolve_module_name(path, module_uri)?;
            dependencies.push(dep_url.clone());

            // Recursively visit the dependency
            self.visit_module(&dep_url, visited, stack, graph)?;
        }

        // Add the module and its dependencies to the graph
        graph.insert(module_uri.clone(), dependencies);

        stack.pop();
        Ok(())
    }

    /// Resolves a module name to a `Url` relative to the current module.
    fn resolve_module_name(&self, name: &str, current_module: &Url) -> Result<Url, Error> {
        current_module.join(name).map_err(|err| DriverError::Url(err).into())
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod path_support {
    use super::*;

    impl Database {
        /// Create a new database tracking the folder at the given path
        /// If the path is a file, the parent directory is tracked
        pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> Self {
            let path = path.as_ref();
            let path = if path.is_dir() {
                path
            } else {
                path.parent().expect("Could not get parent directory")
            };
            Self::from_source(FileSystemSource::new(path))
        }

        /// Create a new database tracking the current working directory
        pub fn from_cwd() -> Self {
            Self::from_path(std::env::current_dir().expect("Could not get current directory"))
        }

        /// Open a file by its path and load it into the database
        pub fn resolve_path<P: AsRef<std::path::Path>>(&mut self, path: P) -> Result<Url, Error> {
            let path = path.as_ref().canonicalize().expect("Could not canonicalize path");
            Ok(Url::from_file_path(path).expect("Could not convert path to URI"))
        }
    }
}