use chalk_rust_parse::ast::*;
use lalrpop_intern::intern;
use errors::*;
use ir;
use std::collections::HashMap;
use std::iter::once;

mod test;

type TypeIds = HashMap<ir::Identifier, ir::ItemId>;
type TypeKinds = HashMap<ir::ItemId, ir::TypeKind>;
type ParameterMap = HashMap<ir::Identifier, usize>;

#[derive(Debug)]
struct Env<'k> {
    type_ids: &'k TypeIds,
    type_kinds: &'k TypeKinds,
    parameter_map: &'k ParameterMap,
}

enum NameLookup {
    Type(ir::ItemId),
    Parameter(usize),
}

const SELF: &str = "Self";

impl<'k> Env<'k> {
    fn lookup(&self, name: Identifier) -> Result<NameLookup> {
        if let Some(k) = self.parameter_map.get(&name.str) {
            return Ok(NameLookup::Parameter(*k));
        }

        if let Some(id) = self.type_ids.get(&name.str) {
            return Ok(NameLookup::Type(*id));
        }

        bail!(ErrorKind::InvalidTypeName(name))
    }

    fn type_kind(&self, id: ir::ItemId) -> &ir::TypeKind {
        &self.type_kinds[&id]
    }
}

pub trait LowerProgram {
    fn lower(&self) -> Result<ir::Program>;
}

impl LowerProgram for Program {
    fn lower(&self) -> Result<ir::Program> {
        let mut type_ids = HashMap::new();
        let mut type_kinds = HashMap::new();
        for (index, item) in self.items.iter().enumerate() {
            let item_id = ir::ItemId { index: index };
            let k = match *item {
                Item::StructDefn(ref d) => d.lower_type_kind()?,
                Item::TraitDefn(ref d) => d.lower_type_kind()?,
                Item::Impl(_) | Item::Goal(_) => continue,
            };
            type_ids.insert(k.name, item_id);
            type_kinds.insert(item_id, k);
        }

        let mut where_clauses = HashMap::new();
        let mut assoc_ty_names = HashMap::new();
        let mut impls = Vec::new();
        let mut goals = Vec::new();
        for (index, item) in self.items.iter().enumerate() {
            let item_id = ir::ItemId { index: index };
            let parameter_map = item.parameter_map();
            let env = Env {
                type_ids: &type_ids,
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
    fn lower_type_kind(&self) -> Result<ir::TypeKind>;
}

trait LowerParameterMap {
    fn synthetic_parameters(&self) -> Option<ir::Identifier>;
    fn declared_parameters(&self) -> &[Identifier];

    fn parameter_map(&self) -> ParameterMap {
        self.synthetic_parameters()
            .into_iter()
            .chain(self.declared_parameters()
                .iter()
                .map(|id| id.str))
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect()
    }
}

impl LowerParameterMap for Item {
    fn synthetic_parameters(&self) -> Option<ir::Identifier> {
        match *self {
            Item::TraitDefn(..) => Some(intern(SELF)),
            Item::StructDefn(..) |
            Item::Impl(..) |
            Item::Goal(..) => None,
        }
    }

    fn declared_parameters(&self) -> &[Identifier] {
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
    fn lower_type_kind(&self) -> Result<ir::TypeKind> {
        Ok(ir::TypeKind {
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
    fn lower_type_kind(&self) -> Result<ir::TypeKind> {
        Ok(ir::TypeKind {
            sort: ir::TypeSort::Trait,
            name: self.name.str,
            parameters: once(intern(SELF)).chain(self.parameters.iter().map(|p| p.str)).collect(),
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
                ir::WhereClause::Implemented(trait_ref.lower(env)?)
            }
            WhereClause::ProjectionEq { ref projection, ref ty } => {
                ir::WhereClause::NormalizeTo(ir::NormalizeTo {
                    projection: projection.lower(env)?,
                    ty: ty.lower(env)?,
                })
            }
        })
    }
}

trait LowerTraitRef {
    fn lower(&self, env: &Env) -> Result<ir::TraitRef>;
}

impl LowerTraitRef for TraitRef {
    fn lower(&self, env: &Env) -> Result<ir::TraitRef> {
        let id = match env.lookup(self.trait_name)? {
            NameLookup::Type(id) => id,
            NameLookup::Parameter(_) => bail!(ErrorKind::NotTrait(self.trait_name)),
        };

        let k = env.type_kind(id);
        if k.sort != ir::TypeSort::Trait {
            bail!(ErrorKind::NotTrait(self.trait_name));
        }

        Ok(ir::TraitRef {
            trait_id: id,
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
                    NameLookup::Type(id) => {
                        let k = env.type_kind(id);
                        if k.parameters.len() > 0 {
                            bail!(ErrorKind::IncorrectNumberOfTypeParameters(name,
                                                                             k.parameters.len(),
                                                                             0))
                        }

                        Ok(ir::Ty::Apply(ir::ApplicationTy {
                            id: id,
                            args: vec![],
                        }))
                    }
                    NameLookup::Parameter(d) => Ok(ir::Ty::Var(d)),
                }
            }

            Ty::Apply { name, ref args } => {
                let id = match env.lookup(name)? {
                    NameLookup::Type(id) => id,
                    NameLookup::Parameter(_) => bail!(ErrorKind::CannotApplyTypeParameter(name)),
                };

                let k = env.type_kind(id);
                if k.parameters.len() != args.len() {
                    bail!(ErrorKind::IncorrectNumberOfTypeParameters(name,
                                                                     k.parameters.len(),
                                                                     args.len()))
                }

                let args = try!(args.iter().map(|t| t.lower(env)).collect());

                Ok(ir::Ty::Apply(ir::ApplicationTy {
                    id: id,
                    args: args,
                }))
            }

            Ty::Projection { ref proj } => Ok(ir::Ty::Projection(proj.lower(env)?)),
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
