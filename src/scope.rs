use std::{collections::HashMap, mem};

use once_cell::sync::Lazy;

use crate::{ast::Node, bytecode::eval_bytecode};

pub const GLOBAL_SCOPE: Lazy<Scope> = Lazy::new(|| Scope::new("<Fang>".to_string(), None));

#[derive(Debug, Clone)]
pub struct Scope {
    pub name: String,
    pub store: HashMap<String, Node>,
    pub functions: HashMap<String, (Vec<Node>, Vec<Node>)>,
    pub parent: Option<Box<Scope>>,
}

impl Scope {
    pub fn new(name: String, parent: Option<Box<Scope>>) -> Self {
        Scope {
            name,
            store: HashMap::new(),
            functions: HashMap::new(),
            parent,
        }
    }

    pub fn declare(&mut self, name: String, val: Node) -> Result<(), String> {
        if self.store.contains_key(&name) {
            return Err("Variable already declared".to_string());
        }

        self.store.insert(name, val);
        Ok(())
    }

    pub fn assign(&mut self, name: String, val: Node) -> Result<(), String> {
        if !self.store.contains_key(&name) {
            return Err(format!(
                "Variable '{}' not declared in scope {}",
                name, self.name
            ));
        }

        if mem::discriminant(self.store.get(&name).unwrap()) != mem::discriminant(&val) {
            return Err("Type mismatch".to_string());
        }

        self.store.insert(name, val);
        Ok(())
    }

    pub fn put_fn(&mut self, name: String, args: Vec<Node>, body: Vec<Node>) -> Result<(), String> {
        if self.functions.contains_key(&name) {
            return Err(format!("Function '{}' already declared", name));
        }

        self.functions.insert(name, (args, body));
        Ok(())
    }

    pub fn get(&self, name: &str) -> Option<&Node> {
        self.store
            .get(name)
            .or(self.parent.as_ref().map(|p| p.get(name)).flatten())
    }

    pub fn get_args(&self, name: &str) -> Option<Vec<Node>> {
        self.functions.get(name).map(|(args, _)| args.clone())
    }

    pub fn call(&self, name: &str, args: Vec<Node>) -> Result<(), String> {
        let func = self.functions.get(name).unwrap();
        let (fn_args, body) = func;

        let mut scope = Scope::new(name.to_string(), Some(Box::new(self.clone())));
        for (arg, val) in fn_args.iter().zip(args.iter()) {
            scope.declare(match arg {
                Node::Identifier { val } => val.clone(),
                Node::TypedVariable { name, .. } => name.clone(),
                _ => return Err("Expected identifier".to_string()),
            
            }, val.clone())?;
        }

        eval_bytecode(body.to_vec(), &mut scope)?;
        Ok(())
    }
}
