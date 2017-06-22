use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use super::*;

const NORMALIZE_CSS: &[u8] = include_bytes!("static/normalize.css");

pub struct DocCompiler<'a> {
    pub out_path: PathBuf,
    pub processor: &'a DocBackend,
}

impl<'a> DocCompiler<'a> {
    fn write_stylesheets(&self) -> Result<()> {
        if !self.out_path.is_dir() {
            debug!("+dir: {}", self.out_path.display());
            fs::create_dir_all(&self.out_path)?;
        }

        let normalize_css = self.out_path.join(NORMALIZE_CSS_NAME);

        debug!("+css: {}", normalize_css.display());
        let mut f = fs::File::create(normalize_css)?;
        f.write_all(NORMALIZE_CSS)?;

        let doc_css = self.out_path.join(DOC_CSS_NAME);

        let content = self.processor.themes.get(self.processor.theme.as_str());

        if let Some(content) = content {
            debug!("+css: {}", doc_css.display());
            let mut f = fs::File::create(doc_css)?;
            f.write_all(content)?;
        } else {
            return Err(format!("no such theme: {}", &self.processor.theme).into());
        }

        Ok(())
    }

    fn write_index(&self, packages: &Vec<RpVersionedPackage>) -> Result<()> {
        let mut out = String::new();

        self.processor
            .write_doc(&mut out, |out| {
                self.processor.write_packages(out, packages, None)?;
                Ok(())
            })?;

        let mut path = self.out_path.join(INDEX);
        path.set_extension(EXT);

        if let Some(parent) = path.parent() {
            if !parent.is_dir() {
                fs::create_dir_all(parent)?;
            }
        }

        debug!("+index: {}", path.display());

        let mut f = fs::File::create(path)?;
        f.write_all(&out.into_bytes())?;

        Ok(())
    }

    fn write_overviews(&self,
                       packages: &Vec<RpVersionedPackage>,
                       files: &mut BTreeMap<&RpVersionedPackage, DocCollector>)
                       -> Result<()> {
        for (package, collector) in files.iter_mut() {
            collector.set_package_title(format!("{}", package));

            {
                let mut out = collector.new_package();
                self.processor.write_packages(&mut out, packages, Some(*package))?;
            }

            {
                let service_bodies = collector.service_bodies.clone();
                let mut out = collector.new_service_overview();
                self.processor.write_service_overview(&mut out, service_bodies)?;
            }

            {
                let decl_bodies = collector.decl_bodies.clone();
                let mut out = collector.new_types_overview();
                self.processor.write_types_overview(&mut out, decl_bodies)?;
            }
        }

        Ok(())
    }
}

impl<'a> Compiler<'a> for DocCompiler<'a> {
    fn compile(&self) -> Result<()> {
        let mut files = self.populate_files()?;
        self.write_stylesheets()?;
        let packages: Vec<_> = files.keys().map(|p| (*p).clone()).collect();
        self.write_index(&packages)?;
        self.write_overviews(&packages, &mut files)?;
        self.write_files(files)?;
        Ok(())
    }
}

impl<'a> PackageProcessor<'a> for DocCompiler<'a> {
    type Out = DocCollector;

    fn ext(&self) -> &str {
        EXT
    }

    fn env(&self) -> &Environment {
        &self.processor.env
    }

    fn out_path(&self) -> &Path {
        &self.out_path
    }

    fn processed_package(&self, package: &RpVersionedPackage) -> RpPackage {
        self.processor.package(package)
    }

    fn default_process(&self, _: &mut Self::Out, type_id: &RpTypeId, _: &RpPos) -> Result<()> {
        let type_id = type_id.clone();
        warn!("Cannot handle: `{:?}", &type_id);
        Ok(())
    }

    fn resolve_full_path(&self, package: &RpPackage) -> Result<PathBuf> {
        let mut full_path = self.out_path().join(self.processor.package_file(package));
        full_path.set_extension(self.ext());
        Ok(full_path)
    }

    fn process_service(&self,
                       out: &mut Self::Out,
                       type_id: &RpTypeId,
                       pos: &RpPos,
                       body: Rc<RpServiceBody>)
                       -> Result<()> {
        self.processor.process_service(out, type_id, pos, body)
    }

    fn process_enum(&self,
                    out: &mut Self::Out,
                    type_id: &RpTypeId,
                    pos: &RpPos,
                    body: Rc<RpEnumBody>)
                    -> Result<()> {
        self.processor.process_enum(out, type_id, pos, body)
    }

    fn process_interface(&self,
                         out: &mut Self::Out,
                         type_id: &RpTypeId,
                         pos: &RpPos,
                         body: Rc<RpInterfaceBody>)
                         -> Result<()> {
        self.processor.process_interface(out, type_id, pos, body)
    }

    fn process_type(&self,
                    out: &mut Self::Out,
                    type_id: &RpTypeId,
                    pos: &RpPos,
                    body: Rc<RpTypeBody>)
                    -> Result<()> {
        self.processor.process_type(out, type_id, pos, body)
    }

    fn process_tuple(&self,
                     out: &mut Self::Out,
                     type_id: &RpTypeId,
                     pos: &RpPos,
                     body: Rc<RpTupleBody>)
                     -> Result<()> {
        self.processor.process_tuple(out, type_id, pos, body)
    }
}
