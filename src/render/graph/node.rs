// use std::rc::Rc;

// pub struct Node {
//     inputs: Vec<(String, Rc<Node>)>,
//     outputs: Vec<(String, Rc<Node>)>,
// }

// pub struct NodeBuilder {
//     inputs: Vec<(String, Rc<Node>)>,
//     outputs: Vec<(String, Rc<Node>)>
// }

// impl NodeBuilder {
//     pub fn new() -> Self {
//         Self {
//             inputs: Vec::new(),
//             outputs: Vec::new()
//         }
//     }
//     pub fn addInput(&mut self, name: &str, node: &Rc<Node>) -> &mut Self {
//         self.inputs.push((name.to_owned(), node.clone()));

//         self
//     }

//     pub fn addOutput(&mut self, name: &str, node: &Rc<Node>) -> &mut Self {
//         self.outputs.push((name.to_owned(), node.clone()));

//         self
//     }

//     pub fn build(self) -> Node {
//         Node {
//             inputs: self.inputs,
//             outputs: self.outputs
//         }
//     }
// }
