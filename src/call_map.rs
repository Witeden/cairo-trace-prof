use std::cell::RefCell;
use std::collections::{hash_map::Entry, HashMap, HashSet};

use serde::Serialize;

use crate::utils::State;

#[derive(Clone, Debug, Serialize)]
pub struct Function {
    pub file: String,
    #[serde(skip)]
    pub name: String,
    pub module: Option<String>,
    pub proper_inst: u32,
    pub cumul_inst: u32,
    pub called: u32,
    pub calls: u32,
    pub callers: HashSet<String>,
    pub callees: HashSet<String>,
}

#[derive(Debug, Serialize)]
pub struct Node {
    pub fun: Function,
    pub callees: HashMap<String, Node>,
}

impl Node {
    pub fn next(fun: RefCell<Function>, cmap: &CallMap) -> Self {
        cmap.cmap.get(&fun.borrow().name).map_or_else(
            || Self {
                fun: fun.borrow().clone(),
                callees: HashMap::default(),
            },
            |fun| {
                let callees = fun
                    .borrow()
                    .callees
                    .iter()
                    .filter(|&name| name != &fun.borrow().name)
                    .map(|name| {
                        (
                            name.to_string(),
                            Self::next(cmap.cmap.get(name).unwrap().clone(), cmap),
                        )
                    })
                    .collect::<HashMap<String, Self>>();
                Self {
                    fun: fun.borrow().clone(),
                    callees,
                }
            },
        )
    }
}

impl Function {
    pub fn new(file: &str, name: &str, module: Option<&str>, caller: &str) -> Self {
        let mut callers = HashSet::new();
        callers.insert(caller.to_owned());
        Self {
            file: file.to_owned(),
            name: name.to_owned(),
            module: module.map(std::borrow::ToOwned::to_owned),
            proper_inst: 1,
            cumul_inst: 0,
            called: 0,
            calls: 0,
            callers,
            callees: HashSet::new(),
        }
    }
}

#[derive(Default, Debug, Serialize)]
pub struct CallMap {
    #[serde(flatten)]
    pub cmap: HashMap<String, RefCell<Function>>,
}

impl CallMap {
    #[must_use]
    pub fn new(program: &HashMap<usize, Vec<String>>, trace: &Vec<State>) -> Self {
        let mut current_fp = usize::MAX;
        let mut current_fun_name: String = String::new();
        let mut cmap = Self::default();
        for State { pc, ap, fp } in trace {
            // If fp = current_fp, we are still in the current function
            if *fp == current_fp {
                cmap.update_inst(&current_fun_name);
            } else {
                // Either we are calling a function, or we are returning from current function
                current_fp = *fp;
                let fun = program
                    .get(&(*pc - 1))
                    .unwrap_or_else(|| panic!("Failed to find {pc} in program instructions"));
                // If ap != fp, the current function is returning
                if *ap != current_fp {
                    let caller = fun.last().expect("Found empty accessible_scopes field.");
                    cmap.update_caller(caller, &current_fun_name);
                    cmap.update_inst(caller);
                    current_fun_name = caller.to_string();
                } else if fun.len() == 2 {
                    cmap.update_callee(&fun[0], &fun[1], None, &current_fun_name);
                    current_fun_name = fun[1].clone();
                } else if fun.len() == 3 {
                    cmap.update_callee(&fun[0], &fun[2], Some(&fun[1]), &current_fun_name);
                    current_fun_name = fun[2].clone();
                } else {
                    panic!("Found accessible scopes field with incorrect number of values.")
                }
            }
        }

        let init_call = cmap
            .root()
            .expect("Failed to find initial_call")
            .borrow()
            .name
            .clone();
        cmap.compute_cumul_inst(&init_call);

        cmap
    }

    fn compute_cumul_inst(&self, fname: &str) -> u32 {
        let mut cumul_inst = 0;
        if let Some(fun) = self.cmap.get(fname) {
            let proper_inst = fun.borrow().proper_inst;
            let callees = &fun.borrow().callees;
            cumul_inst = proper_inst
                + callees
                    .iter()
                    .filter(|&name| name != fname)
                    .map(|name| self.compute_cumul_inst(name))
                    .sum::<u32>();
        }
        if let Some(fun) = self.cmap.get(fname) {
            fun.borrow_mut().cumul_inst = cumul_inst;
        }
        cumul_inst
    }

    pub fn update_callee(&mut self, file: &str, name: &str, module: Option<&str>, caller: &str) {
        match self.cmap.entry(name.to_owned()) {
            Entry::Occupied(o) => {
                let fun = o.into_mut();
                fun.borrow_mut().proper_inst += 1;
                fun.borrow_mut().called += 1;
                fun.borrow_mut().callers.insert(caller.to_owned());
            }
            Entry::Vacant(v) => {
                v.insert(RefCell::new(Function::new(file, name, module, caller)));
            }
        };
    }

    pub fn update_inst(&mut self, name: &str) {
        let fun = self.cmap.get_mut(name).unwrap();
        fun.borrow_mut().proper_inst += 1;
    }

    pub fn update_caller(&mut self, name: &str, callee: &str) {
        if let Some(fun) = self.cmap.get_mut(name) {
            fun.borrow_mut().callees.insert(callee.to_string());
            fun.borrow_mut().calls += 1;
        }
    }

    #[must_use]
    pub fn root(&self) -> Option<RefCell<Function>> {
        self.cmap
            .iter()
            .find(|(_, f)| f.borrow().callers.contains(""))
            .map(|(_, f)| f.clone())
    }

    #[must_use]
    pub fn call_tree(&self) -> Node {
        let root = self.root().expect("Failed to find initial call.");
        Node::next(root, self)
    }
}
