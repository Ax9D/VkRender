use std::collections::{HashMap, HashSet};

use super::renderpass::Renderpass;

struct CompilerArtifact {

}
// struct Compiler {
//    env: HashMap<&'static str>  
// }

use thiserror::Error;
#[derive(Error, Debug)]
pub enum GraphValidationError {
    #[error("No renderpasses output to the window's color buffer! Exactly 1 color output to \"SCREEN_OUTPUT\" is required.")]
    NoScreenOutput
}

fn validate(passes: Vec<Renderpass>) -> Result<(), GraphValidationError>{
   let hasScreenOutput = passes.iter().find(|&pass| {
    pass.colorOutputs().get("SCREEN_OUTPUT").is_some()
   }).is_some();

   if !hasScreenOutput {
       return Err(GraphValidationError::NoScreenOutput);
   }

   let inputs = HashMap::new();
   let outputs = HashMap::new();



   todo!()
}
fn compile(passes: Vec<Renderpass>) {
    
}