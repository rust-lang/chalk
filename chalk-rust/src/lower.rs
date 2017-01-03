use chalk_rust_parse::ast::*;
use ir;
use std::collections::HashMap;

type Result<T> = ::std::result::Result<T, Identifier>;

type TypeKinds = HashMap<ir::Identifier, ir::TypeKind>;

type ParameterMap = HashMap<ir::Identifier, usize>;

struct Env<'k> {
    type_kinds: &'k TypeKinds,
    parameter_map: &'k ParameterMap,
}

enum NameLookup<'k> {
    Type(&'k ir::TypeKind),
    Parameter(usize),
}

impl<'k> Env<'k> {
    fn lookup(&self, name: Identifier) -> Result<NameLookup<'k>> {
        if let Some(k) = self.parameter_map.get(&name.str) {
            return Ok(NameLookup::Parameter(*k));
        }

        if let Some(k) = self.type_kinds.get(&name.str) {
            return Ok(NameLookup::Type(k));
        }

        Err(name)
    }
}

trait LowerProgram {
    fn lower(&self) -> Result<ir::Program>;
}

impl LowerProgram for Program {
    fn lower(&self) -> Result<ir::Program> {
        let type_kinds: TypeKinds = try!(self.items
            .iter()
            .enumerate()
            .flat_map(|(index, item)| {
                let item_id = ir::ItemId { index: index };
                match *item {
                    Item::StructDefn(ref d) => Some(d.lower_type_kind(item_id)),
                    Item::TraitDefn(ref d) => Some(d.lower_type_kind(item_id)),
                    Item::Impl(_) | Item::Goal(_) => None,
                }
            })
            .map(|k_err| k_err.map(|k| (k.name, k)))
            .collect());

        let where_clauses: HashMap<_, _> = try!(self.items
            .iter()
            .enumerate()
            .flat_map(|(index, item)| {
                let item_id = ir::ItemId { index: index };
                match *item {
                    Item::StructDefn(ref d) => {
                        Some(d.lower_where_clauses(&type_kinds).map(|w| (item_id, w)))
                    }
                    Item::TraitDefn(ref d) => {
                        Some(d.lower_where_clauses(&type_kinds).map(|w| (item_id, w)))
                    }
                    Item::Impl(ref d) => {
                        Some(d.lower_where_clauses(&type_kinds).map(|w| (item_id, w)))
                    }
                    Item::Goal(_) => None,
                }
            })
            .collect());

        unimplemented!()
    }
}

trait LowerTypeKind {
    fn lower_type_kind(&self, item_id: ir::ItemId) -> Result<ir::TypeKind>;
}

trait LowerWhereClauses {
    fn parameters(&self) -> &[Identifier];
    fn where_clauses(&self) -> &[WhereClause];

    fn lower_where_clauses(&self, type_kinds: &TypeKinds) -> Result<Vec<ir::WhereClause>> {
        let parameter_map: ParameterMap = self.parameters()
            .iter()
            .enumerate()
            .map(|(index, id)| (id.str, index))
            .collect();

        let env = Env {
            parameter_map: &parameter_map,
            type_kinds: type_kinds,
        };

        self.where_clauses()
            .iter()
            .map(|wc| wc.lower(&env))
            .collect()
    }
}

impl LowerTypeKind for StructDefn {
    fn lower_type_kind(&self, item_id: ir::ItemId) -> Result<ir::TypeKind> {
        Ok(ir::TypeKind {
            id: item_id,
            sort: ir::TypeSort::Struct,
            name: self.name.str,
            parameters: self.parameters.iter().map(|p| p.str).collect(),
        })
    }
}

impl LowerWhereClauses for StructDefn {
    fn parameters(&self) -> &[Identifier] {
        &self.parameters
    }
    fn where_clauses(&self) -> &[WhereClause] {
        &self.where_clauses
    }
}

impl LowerTypeKind for TraitDefn {
    fn lower_type_kind(&self, item_id: ir::ItemId) -> Result<ir::TypeKind> {
        Ok(ir::TypeKind {
            id: item_id,
            sort: ir::TypeSort::Trait,
            name: self.name.str,
            parameters: self.parameters.iter().map(|p| p.str).collect(),
        })
    }
}

impl LowerWhereClauses for TraitDefn {
    fn parameters(&self) -> &[Identifier] {
        &self.parameters
    }
    fn where_clauses(&self) -> &[WhereClause] {
        &self.where_clauses
    }
}

impl LowerWhereClauses for Impl {
    fn parameters(&self) -> &[Identifier] {
        &self.parameters
    }
    fn where_clauses(&self) -> &[WhereClause] {
        &self.where_clauses
    }
}

trait LowerWhereClause {
    fn lower(&self, env: &Env) -> Result<ir::WhereClause>;
}

impl LowerWhereClause for WhereClause {
    fn lower(&self, env: &Env) -> Result<ir::WhereClause> {
        Ok(match *self {
            WhereClause::Implemented { ref trait_ref } => {
                ir::WhereClause::Implemented { trait_ref: trait_ref.lower(env)? }
            }
            WhereClause::ProjectionEq { ref projection, ref ty } => {
                ir::WhereClause::ProjectionEq {
                    projection: projection.lower(env)?,
                    ty: ty.lower(env)?,
                }
            }
        })
    }
}

trait LowerTraitRef {
    fn lower(&self, env: &Env) -> Result<ir::TraitRef>;
}

impl LowerTraitRef for TraitRef {
    fn lower(&self, env: &Env) -> Result<ir::TraitRef> {
        Ok(ir::TraitRef {
            trait_name: self.trait_name.str,
            args: try!(self.args.iter().map(|a| a.lower(env)).collect()),
        })
    }
}

trait LowerProjectionTy {
    fn lower(&self, env: &Env) -> Result<ir::ProjectionTy>;
}

impl LowerProjectionTy for ProjectionTy {
    fn lower(&self, env: &Env) -> Result<ir::ProjectionTy> {
        Ok(ir::ProjectionTy {
            trait_ref: self.trait_ref.lower(env)?,
            name: self.name.str,
        })
    }
}

trait LowerTy {
    fn lower(&self, env: &Env) -> Result<ir::Ty>;
}

impl LowerTy for Ty {
    fn lower(&self, env: &Env) -> Result<ir::Ty> {
        match *self {
            Ty::Id { name } => {
                match env.lookup(name)? {
                    NameLookup::Type(k) => {
                        unimplemented!()
                    }
                    NameLookup::Parameter(d) => {
                        Ok(ir::Ty::Var { depth: d })
                    }
                }
            }

            Ty::Apply { name, ref args } => {
                let k = match env.lookup(name)? {
                    NameLookup::Type(k) => k,
                    NameLookup::Parameter(d) => {
                        unimplemented!()
                    }
                };

                if k.parameters.len() != args.len() {
                    unimplemented!()
                }

                let args = try!(args.iter().map(|t| t.lower(env)).collect());

                Ok(ir::Ty::Apply { name: k.name, args: args })
            }

            Ty::Projection { ref proj } => {
                Ok(ir::Ty::Projection { proj: proj.lower(env)? })
            }
        }
    }
}
