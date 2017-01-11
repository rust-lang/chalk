use cast::Cast;
use chalk_rust_parse::ast::*;
use lalrpop_intern::intern;
use errors::*;
use ir;
use std::collections::HashMap;

mod test;

type TypeIds = HashMap<ir::Identifier, ir::ItemId>;
type TypeKinds = HashMap<ir::ItemId, ir::TypeKind>;
type ParameterMap = HashMap<ir::ParameterKind<ir::Identifier>, usize>;

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

enum LifetimeLookup {
    Parameter(usize),
}

const SELF: &str = "Self";

impl<'k> Env<'k> {
    fn lookup(&self, name: Identifier) -> Result<NameLookup> {
        if let Some(k) = self.parameter_map.get(&ir::ParameterKind::Ty(name.str)) {
            return Ok(NameLookup::Parameter(*k));
        }

        if let Some(id) = self.type_ids.get(&name.str) {
            return Ok(NameLookup::Type(*id));
        }

        bail!(ErrorKind::InvalidTypeName(name))
    }

    fn lookup_lifetime(&self, name: Identifier) -> Result<LifetimeLookup> {
        if let Some(k) = self.parameter_map.get(&ir::ParameterKind::Lifetime(name.str)) {
            return Ok(LifetimeLookup::Parameter(*k));
        }

        bail!("invalid lifetime name: {:?}", name.str);
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
                Item::Impl(_) => continue,
            };
            type_ids.insert(k.name, item_id);
            type_kinds.insert(item_id, k);
        }

        let mut trait_data = HashMap::new();
        let mut impl_data = HashMap::new();
        for (index, item) in self.items.iter().enumerate() {
            let item_id = ir::ItemId { index: index };
            let parameter_map = item.parameter_map();
            let env = Env {
                type_ids: &type_ids,
                type_kinds: &type_kinds,
                parameter_map: &parameter_map,
            };
            match *item {
                Item::StructDefn(ref _d) => {
                    // where_clauses.insert(item_id, d.lower_where_clauses(&env)?);
                }
                Item::TraitDefn(ref d) => {
                    // where_clauses.insert(item_id, d.lower_where_clauses(&env)?);
                    trait_data.insert(item_id, d.lower_trait(&env)?);
                }
                Item::Impl(ref d) => {
                    impl_data.insert(item_id, d.lower_impl(&env)?);
                }
            }
        }

        Ok(ir::Program { type_ids, type_kinds, trait_data, impl_data })
    }
}

trait LowerTypeKind {
    fn lower_type_kind(&self) -> Result<ir::TypeKind>;
}

trait LowerParameterMap {
    fn synthetic_parameters(&self) -> Option<ir::ParameterKind<ir::Identifier>>;
    fn declared_parameters(&self) -> &[ParameterKind];
    fn all_parameters(&self) -> Vec<ir::ParameterKind<ir::Identifier>> {
        self.declared_parameters()
            .iter()
            .map(|id| id.lower())
            .chain(self.synthetic_parameters()) // (*) see above
            .collect()
    }

    fn parameter_map(&self) -> ParameterMap {
        // (*) It is important that the declared parameters come
        // before the subtle parameters in the ordering. This is
        // because of traits, when used as types, only have the first
        // N parameters in their kind (that is, they do not have Self).
        //
        // Note that if `Self` appears in the where-clauses etc, the
        // trait is not object-safe, and hence not supposed to be used
        // as an object. Actually the handling of object types is
        // probably just kind of messed up right now. That's ok.
        self.all_parameters()
            .into_iter()
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect()
    }
}

impl LowerParameterMap for Item {
    fn synthetic_parameters(&self) -> Option<ir::ParameterKind<ir::Identifier>> {
        match *self {
            Item::TraitDefn(ref d) => d.synthetic_parameters(),
            Item::StructDefn(..) |
            Item::Impl(..) => None,
        }
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        match *self {
            Item::TraitDefn(ref d) => d.declared_parameters(),
            Item::StructDefn(ref d) => &d.parameter_kinds,
            Item::Impl(ref d) => &d.parameter_kinds,
        }
    }
}

impl LowerParameterMap for TraitDefn {
   fn synthetic_parameters(&self) -> Option<ir::ParameterKind<ir::Identifier>> {
       Some(ir::ParameterKind::Ty(intern(SELF)))
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}


trait LowerParameterKind {
    fn lower(&self) -> ir::ParameterKind<ir::Identifier>;
}

impl LowerParameterKind for ParameterKind {
    fn lower(&self) -> ir::ParameterKind<ir::Identifier> {
        match *self {
            ParameterKind::Ty(ref n) => ir::ParameterKind::Ty(n.str),
            ParameterKind::Lifetime(ref n) => ir::ParameterKind::Lifetime(n.str),
        }
    }
}

trait LowerWhereClauses {
    fn where_clauses(&self) -> &[WhereClause];

    fn lower_where_clauses(&self, env: &Env) -> Result<Vec<ir::WhereClause>> {
        self.where_clauses().lower(env)
    }
}

impl LowerTypeKind for StructDefn {
    fn lower_type_kind(&self) -> Result<ir::TypeKind> {
        Ok(ir::TypeKind {
            sort: ir::TypeSort::Struct,
            name: self.name.str,
            parameter_kinds: self.parameter_kinds.iter().map(|p| p.lower()).collect(),
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

            // for the purposes of the *type*, ignore `Self`:
            parameter_kinds: self.parameter_kinds.iter().map(|p| p.lower()).collect(),
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

trait LowerWhereClauseVec {
    fn lower(&self, env: &Env) -> Result<Vec<ir::WhereClause>>;
}

impl LowerWhereClauseVec for [WhereClause] {
    fn lower(&self, env: &Env) -> Result<Vec<ir::WhereClause>> {
        self.iter()
            .map(|wc| wc.lower(env))
            .collect()
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
                ir::WhereClause::Normalize(ir::Normalize {
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

        let parameters = self.args.iter().map(|a| Ok(ir::ParameterKind::Ty(a.lower(env)?))).collect::<Result<Vec<_>>>()?;

        Ok(ir::TraitRef {
            trait_id: id,
            parameters: parameters,
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
                        if k.parameter_kinds.len() > 0 {
                            bail!(ErrorKind::IncorrectNumberOfTypeParameters(name,
                                                                             k.parameter_kinds.len(),
                                                                             0))
                        }

                        Ok(ir::Ty::Apply(ir::ApplicationTy {
                            name: ir::TypeName::ItemId(id),
                            parameters: vec![],
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
                if k.parameter_kinds.len() != args.len() {
                    bail!(ErrorKind::IncorrectNumberOfTypeParameters(name,
                                                                     k.parameter_kinds.len(),
                                                                     args.len()))
                }

                let parameters = args.iter().map(|t| Ok(t.lower(env)?)).collect::<Result<Vec<_>>>()?;

                Ok(ir::Ty::Apply(ir::ApplicationTy {
                    name: ir::TypeName::ItemId(id),
                    parameters: parameters,
                }))
            }

            Ty::Projection { ref proj } => Ok(ir::Ty::Projection(proj.lower(env)?)),
        }
    }
}

trait LowerParameter {
    fn lower(&self, env: &Env) -> Result<ir::Parameter>;
}

impl LowerParameter for Parameter {
    fn lower(&self, env: &Env) -> Result<ir::Parameter> {
        match *self {
            Parameter::Ty(ref t) => Ok(ir::ParameterKind::Ty(t.lower(env)?)),
            Parameter::Lifetime(ref l) => Ok(ir::ParameterKind::Lifetime(l.lower(env)?)),
        }
    }
}

trait LowerLifetime {
    fn lower(&self, env: &Env) -> Result<ir::Lifetime>;
}

impl LowerLifetime for Lifetime {
    fn lower(&self, env: &Env) -> Result<ir::Lifetime> {
        match *self {
            Lifetime::Id { name } => {
                match env.lookup_lifetime(name)? {
                    LifetimeLookup::Parameter(d) => Ok(ir::Lifetime::Var(d))
                }
            }
        }
    }
}

trait LowerImpl {
    fn lower_impl(&self, env: &Env) -> Result<ir::ImplData>;
}

impl LowerImpl for Impl {
    fn lower_impl(&self, env: &Env) -> Result<ir::ImplData> {
        Ok(ir::ImplData {
            trait_ref: self.trait_ref.lower(env)?,
            parameter_kinds: self.parameter_kinds.iter().map(|p| p.lower()).collect(),
            assoc_ty_values: try!(self.assoc_ty_values.iter().map(|v| v.lower(env)).collect()),
            where_clauses: self.lower_where_clauses(&env)?,
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

trait LowerTrait {
    fn lower_trait(&self, env: &Env) -> Result<ir::TraitData>;
}

impl LowerTrait for TraitDefn {
    fn lower_trait(&self, env: &Env) -> Result<ir::TraitData> {
        Ok(ir::TraitData {
            parameter_kinds: self.all_parameters(),
            where_clauses: self.lower_where_clauses(&env)?,
            assoc_ty_names: self.assoc_ty_names.iter().map(|a| a.str).collect(),
        })
    }
}

pub trait LowerGoal<A> {
    fn lower(&self, arg: &A) -> Result<Box<ir::Goal>>;
}

impl LowerGoal<ir::Program> for Goal {
    fn lower(&self, program: &ir::Program) -> Result<Box<ir::Goal>> {
        let env = Env {
            type_ids: &program.type_ids,
            type_kinds: &program.type_kinds,
            parameter_map: &HashMap::new()
        };

        self.lower(&env)
    }
}

impl<'k> LowerGoal<Env<'k>> for Goal {
    fn lower(&self, env: &Env<'k>) -> Result<Box<ir::Goal>> {
        match *self {
            Goal::ForAll(ref ids, ref g) =>
                g.lower_quantified(env, ir::QuantifierKind::ForAll, ids),
            Goal::Exists(ref ids, ref g) =>
                g.lower_quantified(env, ir::QuantifierKind::Exists, ids),
            Goal::Implies(ref wc, ref g) =>
                Ok(Box::new(ir::Goal::Implies(wc.lower(env)?, g.lower(env)?))),
            Goal::And(ref g1, ref g2) =>
                Ok(Box::new(ir::Goal::And(g1.lower(env)?, g2.lower(env)?))),
            Goal::Leaf(ref wc) =>
                Ok(Box::new(ir::Goal::Leaf(wc.lower(env)?.cast()))),
        }
    }
}

trait LowerQuantifiedGoal {
    fn lower_quantified(&self,
                        env: &Env,
                        quantifier_kind: ir::QuantifierKind,
                        parameter_kinds: &[ParameterKind])
                        -> Result<Box<ir::Goal>>;
}

impl LowerQuantifiedGoal for Goal {
    fn lower_quantified(&self,
                        env: &Env,
                        quantifier_kind: ir::QuantifierKind,
                        parameter_kinds: &[ParameterKind])
                        -> Result<Box<ir::Goal>>
    {
        if parameter_kinds.is_empty() {
            return self.lower(env);
        }

        let next_id = env.parameter_map.len();
        let mut parameter_map = env.parameter_map.clone();
        parameter_map.insert(parameter_kinds[0].lower(), next_id);
        let parameter_kind = match parameter_kinds[0] {
            ParameterKind::Ty(_) => ir::ParameterKind::Ty(()),
            ParameterKind::Lifetime(_) => ir::ParameterKind::Lifetime(()),
        };
        let quantified_env = &Env { parameter_map: &parameter_map, ..*env };
        let subgoal = self.lower_quantified(quantified_env, quantifier_kind, &parameter_kinds[1..])?;
        Ok(Box::new(ir::Goal::Quantified(quantifier_kind, parameter_kind, subgoal)))
    }
}
