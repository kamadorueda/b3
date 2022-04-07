// SPDX-FileCopyrightText: 2022 Kevin Amado <kamadorueda@gmail.com>
//
// SPDX-License-Identifier: AGPL-3.0-only

use std::collections::HashMap;
use std::collections::LinkedList;

use nixel::ast::Attribute;
use nixel::ast::AttributePath;
use nixel::ast::Binding as NixelBinding;
use nixel::ast::AST;

pub(crate) struct Bindings {
    pub(crate) bindings: HashMap<String, Binding>,
}

#[derive(Debug)]
pub(crate) struct Binding {
    pub(crate) ast:       AST,
    pub(crate) inherited: bool,
}

impl Bindings {
    pub(crate) fn new(bindings: LinkedList<NixelBinding>) -> Bindings {
        let mut value_bindings = HashMap::new();

        for binding in bindings {
            add_binding(&mut value_bindings, binding);
        }

        Bindings { bindings: value_bindings }
    }
}

fn add_binding(bindings: &mut HashMap<String, Binding>, binding: NixelBinding) {
    match binding {
        NixelBinding::KeyValue(mut attribute_path, ast) => {
            let attribute = attribute_path.attributes.pop_front().unwrap();

            match attribute {
                Attribute::Raw { content, .. } => {
                    bindings.insert(
                        content,
                        Binding {
                            ast:       if attribute_path.attributes.is_empty() {
                                *ast
                            } else {
                                AST::__Bindings(LinkedList::from([
                                    NixelBinding::KeyValue(
                                        AttributePath {
                                            attributes: attribute_path
                                                .attributes,
                                        },
                                        ast,
                                    ),
                                ]))
                            },
                            inherited: false,
                        },
                    );
                }
                _ => todo!(),
            }
        }
        _ => todo!(),
    }
}
