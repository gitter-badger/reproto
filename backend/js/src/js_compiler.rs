use super::*;
use std::path::{Path, PathBuf};
use std::rc::Rc;

pub struct JsCompiler<'a> {
    pub out_path: PathBuf,
    pub backend: &'a JsBackend,
}

impl<'a> JsCompiler<'a> {
    pub fn compile(&self) -> Result<()> {
        let files = self.populate_files()?;
        self.write_files(files)
    }
}

impl<'a> PackageProcessor<'a> for JsCompiler<'a> {
    type Out = JsFileSpec;

    fn ext(&self) -> &str {
        EXT
    }

    fn env(&self) -> &Environment {
        &self.backend.env
    }

    fn out_path(&self) -> &Path {
        &self.out_path
    }

    fn processed_package(&self, package: &RpVersionedPackage) -> RpPackage {
        self.backend.package(package)
    }

    fn process_tuple(
        &self,
        out: &mut Self::Out,
        name: &RpName,
        pos: &Pos,
        body: Rc<RpTupleBody>,
    ) -> Result<()> {
        self.backend.process_tuple(out, name, pos, body)
    }

    fn process_enum(
        &self,
        out: &mut Self::Out,
        name: &RpName,
        pos: &Pos,
        body: Rc<RpEnumBody>,
    ) -> Result<()> {
        self.backend.process_enum(out, name, pos, body)
    }


    fn process_type(
        &self,
        out: &mut Self::Out,
        name: &RpName,
        pos: &Pos,
        body: Rc<RpTypeBody>,
    ) -> Result<()> {
        self.backend.process_type(out, name, pos, body)
    }

    fn process_interface(
        &self,
        out: &mut Self::Out,
        name: &RpName,
        pos: &Pos,
        body: Rc<RpInterfaceBody>,
    ) -> Result<()> {
        self.backend.process_interface(out, name, pos, body)
    }
}
