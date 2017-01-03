use chalk_rust_parse::ast::*;
use errors::*;
use ir;
use std::collections::HashMap;

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

        bail!(ErrorKind::InvalidTypeName(name))
    }
}

pub trait LowerProgram {
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

        let mut where_clauses = HashMap::new();
        let mut assoc_ty_names = HashMap::new();
        let mut impls = Vec::new();
        let mut goals = Vec::new();
        for (index, item) in self.items.iter().enumerate() {
            let item_id = ir::ItemId { index: index };
            let parameter_map = item.parameter_map();
            let env = Env {
                type_kinds: &type_kinds,
                parameter_map: &parameter_map,
            };
            match *item {
                Item::StructDefn(ref d) => {
                    where_clauses.insert(item_id, d.lower_where_clauses(&env)?);
                }
                Item::TraitDefn(ref d) => {
                    let names = d.assoc_ty_names.iter().map(|a| a.str).collect();
                    assoc_ty_names.insert(item_id, names);
                    where_clauses.insert(item_id, d.lower_where_clauses(&env)?);
                }
                Item::Impl(ref d) => {
                    impls.push(d.lower_impl(item_id, &env)?);
                    where_clauses.insert(item_id, d.lower_where_clauses(&env)?);
                }
                Item::Goal(ref d) => {
                    goals.push(d.lower(&env)?);
                }
            }
        }

        Ok(ir::Program {
            type_kinds: type_kinds,
            where_clauses: where_clauses,
            goals: goals,
            assoc_ty_names: assoc_ty_names,
            impls: impls,
        })
    }
}

trait LowerTypeKind {
    fn lower_type_kind(&self, item_id: ir::ItemId) -> Result<ir::TypeKind>;
}

trait LowerParameterMap {
    fn parameters(&self) -> &[Identifier];

    fn parameter_map(&self) -> ParameterMap {
        self.parameters()
            .iter()
            .enumerate()
            .map(|(index, id)| (id.str, index))
            .collect()
    }
}

impl LowerParameterMap for Item {
    fn parameters(&self) -> &[Identifier] {
        match *self {
            Item::StructDefn(ref d) => &d.parameters,
            Item::TraitDefn(ref d) => &d.parameters,
            Item::Impl(ref d) => &d.parameters,
            Item::Goal(..) => &[],
        }
    }
}

trait LowerWhereClauses {
    fn where_clauses(&self) -> &[WhereClause];

    fn lower_where_clauses(&self, env: &Env) -> Result<Vec<ir::WhereClause>> {
        self.where_clauses()
            .iter()
            .map(|wc| wc.lower(env))
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
    fn where_clauses(&self) -> &[WhereClause] {
        &self.where_clauses
    }
}

impl LowerWhereClauses for Impl {
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
        let k = match env.lookup(self.trait_name)? {
            NameLookup::Type(k) => k,
            NameLookup::Parameter(_) => bail!(ErrorKind::NotTrait(self.trait_name)),
        };

        if k.sort != ir::TypeSort::Trait {
            bail!(ErrorKind::NotTrait(self.trait_name));
        }

        Ok(ir::TraitRef {
            trait_id: k.id,
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
                        if k.parameters.len() > 0 {
                            bail!(ErrorKind::IncorrectNumberOfTypeParameters(name,
                                                                             k.parameters.len(),
                                                                             0))
                        }

                        Ok(ir::Ty::Apply {
                            id: k.id,
                            args: vec![],
                        })
                    }
                    NameLookup::Parameter(d) => Ok(ir::Ty::Var { depth: d }),
                }
            }

            Ty::Apply { name, ref args } => {
                let k = match env.lookup(name)? {
                    NameLookup::Type(k) => k,
                    NameLookup::Parameter(_) => bail!(ErrorKind::CannotApplyTypeParameter(name)),
                };

                if k.parameters.len() != args.len() {
                    bail!(ErrorKind::IncorrectNumberOfTypeParameters(name,
                                                                     k.parameters.len(),
                                                                     args.len()))
                }

                let args = try!(args.iter().map(|t| t.lower(env)).collect());

                Ok(ir::Ty::Apply {
                    id: k.id,
                    args: args,
                })
            }

            Ty::Projection { ref proj } => Ok(ir::Ty::Projection { proj: proj.lower(env)? }),
        }
    }
}

trait LowerImpl {
    fn lower_impl(&self, item_id: ir::ItemId, env: &Env) -> Result<ir::Impl>;
}

impl LowerImpl for Impl {
    fn lower_impl(&self, item_id: ir::ItemId, env: &Env) -> Result<ir::Impl> {
        Ok(ir::Impl {
            id: item_id,
            trait_ref: self.trait_ref.lower(env)?,
            parameters: self.parameters.iter().map(|p| p.str).collect(),
            assoc_ty_values: try!(self.assoc_ty_values.iter().map(|v| v.lower(env)).collect()),
        })
    }
}

trait LowerAssocTyValue {
    fn lower(&self, env: &Env) -> Result<ir::AssocTyValue>;
}

impl LowerAssocTyValue for AssocTyValue {
    fn lower(&self, env: &Env) -> Result<ir::AssocTyValue> {
        Ok(ir::AssocTyValue {
            name: self.name.str,
            value: self.value.lower(env)?,
        })
    }
}
