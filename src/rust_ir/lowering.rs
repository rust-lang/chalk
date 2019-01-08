use std::collections::BTreeMap;

use chalk_parse::ast::*;
use lalrpop_intern::intern;

use chalk_ir;
use chalk_ir::cast::{Cast, Caster};
use chalk_solve::solve::SolverChoice;
use crate::errors::*;
use itertools::Itertools;
use crate::rust_ir::{self, Anonymize, ToParameter};

mod test;

type TypeIds = BTreeMap<chalk_ir::Identifier, chalk_ir::ItemId>;
type TypeKinds = BTreeMap<chalk_ir::ItemId, rust_ir::TypeKind>;
type AssociatedTyInfos = BTreeMap<(chalk_ir::ItemId, chalk_ir::Identifier), AssociatedTyInfo>;
type ParameterMap = BTreeMap<chalk_ir::ParameterKind<chalk_ir::Identifier>, usize>;

#[derive(Clone, Debug)]
struct Env<'k> {
    type_ids: &'k TypeIds,
    type_kinds: &'k TypeKinds,
    associated_ty_infos: &'k AssociatedTyInfos,
    /// Parameter identifiers are used as keys, therefore
    /// all indentifiers in an environment must be unique (no shadowing).
    parameter_map: ParameterMap,
}

#[derive(Debug, PartialEq, Eq)]
struct AssociatedTyInfo {
    id: chalk_ir::ItemId,
    addl_parameter_kinds: Vec<chalk_ir::ParameterKind<chalk_ir::Identifier>>,
}

enum NameLookup {
    Type(chalk_ir::ItemId),
    Parameter(usize),
}

enum LifetimeLookup {
    Parameter(usize),
}

const SELF: &str = "Self";

impl<'k> Env<'k> {
    fn lookup(&self, name: Identifier) -> Result<NameLookup> {
        if let Some(k) = self
            .parameter_map
            .get(&chalk_ir::ParameterKind::Ty(name.str))
        {
            return Ok(NameLookup::Parameter(*k));
        }

        if let Some(id) = self.type_ids.get(&name.str) {
            return Ok(NameLookup::Type(*id));
        }

        bail!(ErrorKind::InvalidTypeName(name))
    }

    fn lookup_lifetime(&self, name: Identifier) -> Result<LifetimeLookup> {
        if let Some(k) = self
            .parameter_map
            .get(&chalk_ir::ParameterKind::Lifetime(name.str))
        {
            return Ok(LifetimeLookup::Parameter(*k));
        }

        bail!("invalid lifetime name: {:?}", name.str);
    }

    fn type_kind(&self, id: chalk_ir::ItemId) -> &rust_ir::TypeKind {
        &self.type_kinds[&id]
    }

    /// Introduces new parameters, shifting the indices of existing
    /// parameters to accommodate them. The indices of the new binders
    /// will be assigned in order as they are iterated.
    fn introduce<I>(&self, binders: I) -> Result<Self>
    where
        I: IntoIterator<Item = chalk_ir::ParameterKind<chalk_ir::Identifier>>,
        I::IntoIter: ExactSizeIterator,
    {
        let binders = binders.into_iter().enumerate().map(|(i, k)| (k, i));
        let len = binders.len();
        let parameter_map: ParameterMap = self
            .parameter_map
            .iter()
            .map(|(&k, &v)| (k, v + len))
            .chain(binders)
            .collect();
        if parameter_map.len() != self.parameter_map.len() + len {
            bail!("duplicate or shadowed parameters");
        }
        Ok(Env {
            parameter_map,
            ..*self
        })
    }

    fn in_binders<I, T, OP>(&self, binders: I, op: OP) -> Result<chalk_ir::Binders<T>>
    where
        I: IntoIterator<Item = chalk_ir::ParameterKind<chalk_ir::Identifier>>,
        I::IntoIter: ExactSizeIterator,
        OP: FnOnce(&Self) -> Result<T>,
    {
        let binders: Vec<_> = binders.into_iter().collect();
        let env = self.introduce(binders.iter().cloned())?;
        Ok(chalk_ir::Binders {
            binders: binders.anonymize(),
            value: op(&env)?,
        })
    }
}

pub trait LowerProgram {
    /// Lowers from a Program AST to the internal IR for a program.
    fn lower(&self, solver_choice: SolverChoice) -> Result<rust_ir::Program>;
}

impl LowerProgram for Program {
    fn lower(&self, solver_choice: SolverChoice) -> Result<rust_ir::Program> {
        let mut index = 0;
        let mut next_item_id = || -> chalk_ir::ItemId {
            let i = index;
            index += 1;
            chalk_ir::ItemId { index: i }
        };

        // Make a vector mapping each thing in `items` to an id,
        // based just on its position:
        let item_ids: Vec<_> = self.items.iter().map(|_| next_item_id()).collect();

        // Create ids for associated types
        let mut associated_ty_infos = BTreeMap::new();
        for (item, &item_id) in self.items.iter().zip(&item_ids) {
            if let Item::TraitDefn(ref d) = *item {
                if d.flags.auto && !d.assoc_ty_defns.is_empty() {
                    bail!("auto trait cannot define associated types");
                }
                for defn in &d.assoc_ty_defns {
                    let addl_parameter_kinds = defn.all_parameters();
                    let info = AssociatedTyInfo {
                        id: next_item_id(),
                        addl_parameter_kinds,
                    };
                    associated_ty_infos.insert((item_id, defn.name.str), info);
                }
            }
        }

        let mut type_ids = BTreeMap::new();
        let mut type_kinds = BTreeMap::new();
        for (item, &item_id) in self.items.iter().zip(&item_ids) {
            let k = match *item {
                Item::StructDefn(ref d) => d.lower_type_kind()?,
                Item::TraitDefn(ref d) => d.lower_type_kind()?,
                Item::Impl(_) => continue,
                Item::Clause(_) => continue,
            };
            type_ids.insert(k.name, item_id);
            type_kinds.insert(item_id, k);
        }

        let mut struct_data = BTreeMap::new();
        let mut trait_data = BTreeMap::new();
        let mut impl_data = BTreeMap::new();
        let mut associated_ty_data = BTreeMap::new();
        let mut custom_clauses = Vec::new();
        let mut lang_items = BTreeMap::new();
        for (item, &item_id) in self.items.iter().zip(&item_ids) {
            let empty_env = Env {
                type_ids: &type_ids,
                type_kinds: &type_kinds,
                associated_ty_infos: &associated_ty_infos,
                parameter_map: BTreeMap::new(),
            };

            match *item {
                Item::StructDefn(ref d) => {
                    struct_data.insert(item_id, d.lower_struct(item_id, &empty_env)?);
                }
                Item::TraitDefn(ref d) => {
                    trait_data.insert(item_id, d.lower_trait(item_id, &empty_env)?);

                    for defn in &d.assoc_ty_defns {
                        let info = &associated_ty_infos[&(item_id, defn.name.str)];

                        let mut parameter_kinds = defn.all_parameters();
                        parameter_kinds.extend(d.all_parameters());
                        let env = empty_env.introduce(parameter_kinds.clone())?;

                        associated_ty_data.insert(
                            info.id,
                            rust_ir::AssociatedTyDatum {
                                trait_id: item_id,
                                id: info.id,
                                name: defn.name.str,
                                parameter_kinds: parameter_kinds,
                                bounds: defn.bounds.lower(&env)?,
                                where_clauses: defn.where_clauses.lower(&env)?,
                            },
                        );
                    }

                    if d.flags.deref {
                        use std::collections::btree_map::Entry::*;
                        match lang_items.entry(rust_ir::LangItem::DerefTrait) {
                            Vacant(entry) => {
                                entry.insert(item_id);
                            }
                            Occupied(_) => {
                                bail!(ErrorKind::DuplicateLangItem(rust_ir::LangItem::DerefTrait))
                            }
                        }
                    }
                }
                Item::Impl(ref d) => {
                    impl_data.insert(item_id, d.lower_impl(&empty_env)?);
                }
                Item::Clause(ref clause) => {
                    custom_clauses.extend(clause.lower_clause(&empty_env)?);
                }
            }
        }

        let mut program = rust_ir::Program {
            type_ids,
            type_kinds,
            struct_data,
            trait_data,
            impl_data,
            associated_ty_data,
            custom_clauses,
            lang_items,
            default_impl_data: Vec::new(),
        };

        program.add_default_impls();
        program.record_specialization_priorities(solver_choice)?;
        program.verify_well_formedness(solver_choice)?;
        program.perform_orphan_check(solver_choice)?;
        Ok(program)
    }
}

trait LowerTypeKind {
    fn lower_type_kind(&self) -> Result<rust_ir::TypeKind>;
}

trait LowerParameterMap {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>>;
    fn declared_parameters(&self) -> &[ParameterKind];
    fn all_parameters(&self) -> Vec<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        self.synthetic_parameters()
            .into_iter()
            .chain(self.declared_parameters().iter().map(|id| id.lower()))
            .collect()

        /* TODO: switch to this ordering, but adjust *all* the code to match

        self.declared_parameters()
            .iter()
            .map(|id| id.lower())
            .chain(self.synthetic_parameters()) // (*) see below
            .collect()
         */
    }

    fn parameter_refs(&self) -> Vec<chalk_ir::Parameter> {
        self.all_parameters()
            .anonymize()
            .iter()
            .zip(0..)
            .map(|p| p.to_parameter())
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
            .zip(0..)
            .collect()
    }
}

impl LowerParameterMap for StructDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for Impl {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for AssocTyDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for AssocTyValue {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for TraitDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        Some(chalk_ir::ParameterKind::Ty(intern(SELF)))
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for Clause {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<chalk_ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

trait LowerParameterKind {
    fn lower(&self) -> chalk_ir::ParameterKind<chalk_ir::Identifier>;
}

impl LowerParameterKind for ParameterKind {
    fn lower(&self) -> chalk_ir::ParameterKind<chalk_ir::Identifier> {
        match *self {
            ParameterKind::Ty(ref n) => chalk_ir::ParameterKind::Ty(n.str),
            ParameterKind::Lifetime(ref n) => chalk_ir::ParameterKind::Lifetime(n.str),
        }
    }
}

trait LowerWhereClauses {
    fn where_clauses(&self) -> &[QuantifiedWhereClause];

    fn lower_where_clauses(&self, env: &Env) -> Result<Vec<chalk_ir::QuantifiedWhereClause>> {
        self.where_clauses().lower(env)
    }
}

impl LowerTypeKind for StructDefn {
    fn lower_type_kind(&self) -> Result<rust_ir::TypeKind> {
        Ok(rust_ir::TypeKind {
            sort: rust_ir::TypeSort::Struct,
            name: self.name.str,
            binders: chalk_ir::Binders {
                binders: self.all_parameters().anonymize(),
                value: (),
            },
        })
    }
}

impl LowerWhereClauses for StructDefn {
    fn where_clauses(&self) -> &[QuantifiedWhereClause] {
        &self.where_clauses
    }
}

impl LowerTypeKind for TraitDefn {
    fn lower_type_kind(&self) -> Result<rust_ir::TypeKind> {
        let binders: Vec<_> = self.parameter_kinds.iter().map(|p| p.lower()).collect();
        Ok(rust_ir::TypeKind {
            sort: rust_ir::TypeSort::Trait,
            name: self.name.str,
            binders: chalk_ir::Binders {
                // for the purposes of the *type*, ignore `Self`:
                binders: binders.anonymize(),
                value: (),
            },
        })
    }
}

impl LowerWhereClauses for TraitDefn {
    fn where_clauses(&self) -> &[QuantifiedWhereClause] {
        &self.where_clauses
    }
}

impl LowerWhereClauses for Impl {
    fn where_clauses(&self) -> &[QuantifiedWhereClause] {
        &self.where_clauses
    }
}

trait LowerWhereClauseVec {
    fn lower(&self, env: &Env) -> Result<Vec<chalk_ir::QuantifiedWhereClause>>;
}

impl LowerWhereClauseVec for [QuantifiedWhereClause] {
    fn lower(&self, env: &Env) -> Result<Vec<chalk_ir::QuantifiedWhereClause>> {
        self.iter()
            .flat_map(|wc| match wc.lower(env) {
                Ok(v) => v.into_iter().map(Ok).collect(),
                Err(e) => vec![Err(e)],
            }).collect()
    }
}

trait LowerWhereClause<T> {
    /// Lower from an AST `where` clause to an internal IR.
    /// Some AST `where` clauses can lower to multiple ones, this is why we return a `Vec`.
    /// As for now, this is the only the case for `where T: Foo<Item = U>` which lowers to
    /// `Implemented(T: Foo)` and `ProjectionEq(<T as Foo>::Item = U)`.
    fn lower(&self, env: &Env) -> Result<Vec<T>>;
}

impl LowerWhereClause<chalk_ir::WhereClause> for WhereClause {
    fn lower(&self, env: &Env) -> Result<Vec<chalk_ir::WhereClause>> {
        let where_clauses = match self {
            WhereClause::Implemented { trait_ref } => {
                vec![chalk_ir::WhereClause::Implemented(trait_ref.lower(env)?)]
            }
            WhereClause::ProjectionEq { projection, ty } => vec![
                chalk_ir::WhereClause::ProjectionEq(chalk_ir::ProjectionEq {
                    projection: projection.lower(env)?,
                    ty: ty.lower(env)?,
                }),
                chalk_ir::WhereClause::Implemented(projection.trait_ref.lower(env)?),
            ],
        };
        Ok(where_clauses)
    }
}

impl LowerWhereClause<chalk_ir::QuantifiedWhereClause> for QuantifiedWhereClause {
    fn lower(&self, env: &Env) -> Result<Vec<chalk_ir::QuantifiedWhereClause>> {
        let parameter_kinds = self.parameter_kinds.iter().map(|pk| pk.lower());
        let binders = env.in_binders(parameter_kinds, |env| Ok(self.where_clause.lower(env)?))?;
        Ok(binders.into_iter().collect())
    }
}

trait LowerDomainGoal {
    fn lower(&self, env: &Env) -> Result<Vec<chalk_ir::DomainGoal>>;
}

impl LowerDomainGoal for DomainGoal {
    fn lower(&self, env: &Env) -> Result<Vec<chalk_ir::DomainGoal>> {
        let goals = match self {
            DomainGoal::Holds { where_clause } => {
                where_clause.lower(env)?.into_iter().casted().collect()
            }
            DomainGoal::Normalize { projection, ty } => {
                vec![chalk_ir::DomainGoal::Normalize(chalk_ir::Normalize {
                    projection: projection.lower(env)?,
                    ty: ty.lower(env)?,
                })]
            }
            DomainGoal::TyWellFormed { ty } => vec![chalk_ir::DomainGoal::WellFormed(
                chalk_ir::WellFormed::Ty(ty.lower(env)?),
            )],
            DomainGoal::TraitRefWellFormed { trait_ref } => vec![chalk_ir::DomainGoal::WellFormed(
                chalk_ir::WellFormed::Trait(trait_ref.lower(env)?),
            )],
            DomainGoal::TyFromEnv { ty } => vec![chalk_ir::DomainGoal::FromEnv(
                chalk_ir::FromEnv::Ty(ty.lower(env)?),
            )],
            DomainGoal::TraitRefFromEnv { trait_ref } => vec![chalk_ir::DomainGoal::FromEnv(
                chalk_ir::FromEnv::Trait(trait_ref.lower(env)?),
            )],
            DomainGoal::TraitInScope { trait_name } => {
                let id = match env.lookup(*trait_name)? {
                    NameLookup::Type(id) => id,
                    NameLookup::Parameter(_) => bail!(ErrorKind::NotTrait(*trait_name)),
                };

                if env.type_kind(id).sort != rust_ir::TypeSort::Trait {
                    bail!(ErrorKind::NotTrait(*trait_name));
                }

                vec![chalk_ir::DomainGoal::InScope(id)]
            }
            DomainGoal::Derefs { source, target } => {
                vec![chalk_ir::DomainGoal::Derefs(chalk_ir::Derefs {
                    source: source.lower(env)?,
                    target: target.lower(env)?,
                })]
            }
            DomainGoal::IsLocal { ty } => vec![chalk_ir::DomainGoal::IsLocal(ty.lower(env)?)],
            DomainGoal::IsUpstream { ty } => vec![chalk_ir::DomainGoal::IsUpstream(ty.lower(env)?)],
            DomainGoal::IsFullyVisible { ty } => {
                vec![chalk_ir::DomainGoal::IsFullyVisible(ty.lower(env)?)]
            }
            DomainGoal::LocalImplAllowed { trait_ref } => {
                vec![chalk_ir::DomainGoal::LocalImplAllowed(
                    trait_ref.lower(env)?,
                )]
            }
            DomainGoal::Compatible => vec![chalk_ir::DomainGoal::Compatible(())],
            DomainGoal::DownstreamType { ty } => {
                vec![chalk_ir::DomainGoal::DownstreamType(ty.lower(env)?)]
            }
        };
        Ok(goals)
    }
}

trait LowerLeafGoal {
    fn lower(&self, env: &Env) -> Result<Vec<chalk_ir::LeafGoal>>;
}

impl LowerLeafGoal for LeafGoal {
    fn lower(&self, env: &Env) -> Result<Vec<chalk_ir::LeafGoal>> {
        let goals = match self {
            LeafGoal::DomainGoal { goal } => goal
                .lower(env)?
                .into_iter()
                .map(|goal| chalk_ir::LeafGoal::DomainGoal(goal))
                .collect(),
            LeafGoal::UnifyTys { a, b } => vec![
                chalk_ir::EqGoal {
                    a: chalk_ir::ParameterKind::Ty(a.lower(env)?),
                    b: chalk_ir::ParameterKind::Ty(b.lower(env)?),
                }.cast(),
            ],
            LeafGoal::UnifyLifetimes { ref a, ref b } => vec![
                chalk_ir::EqGoal {
                    a: chalk_ir::ParameterKind::Lifetime(a.lower(env)?),
                    b: chalk_ir::ParameterKind::Lifetime(b.lower(env)?),
                }.cast(),
            ],
        };
        Ok(goals)
    }
}

trait LowerStructDefn {
    fn lower_struct(&self, item_id: chalk_ir::ItemId, env: &Env) -> Result<rust_ir::StructDatum>;
}

impl LowerStructDefn for StructDefn {
    fn lower_struct(&self, item_id: chalk_ir::ItemId, env: &Env) -> Result<rust_ir::StructDatum> {
        let binders = env.in_binders(self.all_parameters(), |env| {
            let self_ty = chalk_ir::ApplicationTy {
                name: chalk_ir::TypeName::ItemId(item_id),
                parameters: self
                    .all_parameters()
                    .anonymize()
                    .iter()
                    .zip(0..)
                    .map(|p| p.to_parameter())
                    .collect(),
            };

            if self.flags.fundamental && self_ty.len_type_parameters() != 1 {
                bail!("Only fundamental types with a single parameter are supported");
            }

            let fields: Result<_> = self.fields.iter().map(|f| f.ty.lower(env)).collect();
            let where_clauses = self.lower_where_clauses(env)?;

            Ok(rust_ir::StructDatumBound {
                self_ty,
                fields: fields?,
                where_clauses,
                flags: rust_ir::StructFlags {
                    upstream: self.flags.upstream,
                    fundamental: self.flags.fundamental,
                },
            })
        })?;

        Ok(rust_ir::StructDatum { binders })
    }
}

fn check_type_kinds<A: Kinded, B: Kinded>(msg: &str, expected: &A, actual: &B) -> Result<()> {
    let expected_kind = expected.kind();
    let actual_kind = actual.kind();
    if expected_kind != actual_kind {
        bail!("{}: expected {}, found {}", msg, expected_kind, actual_kind);
    } else {
        Ok(())
    }
}

trait LowerTraitRef {
    fn lower(&self, env: &Env) -> Result<chalk_ir::TraitRef>;
}

impl LowerTraitRef for TraitRef {
    fn lower(&self, env: &Env) -> Result<chalk_ir::TraitRef> {
        let without_self = TraitBound {
            trait_name: self.trait_name,
            args_no_self: self.args.iter().cloned().skip(1).collect(),
        }.lower(env)?;

        let self_parameter = self.args[0].lower(env)?;
        Ok(without_self.as_trait_ref(self_parameter.ty().unwrap()))
    }
}

trait LowerTraitBound {
    fn lower(&self, env: &Env) -> Result<rust_ir::TraitBound>;
}

impl LowerTraitBound for TraitBound {
    fn lower(&self, env: &Env) -> Result<rust_ir::TraitBound> {
        let id = match env.lookup(self.trait_name)? {
            NameLookup::Type(id) => id,
            NameLookup::Parameter(_) => bail!(ErrorKind::NotTrait(self.trait_name)),
        };

        let k = env.type_kind(id);
        if k.sort != rust_ir::TypeSort::Trait {
            bail!(ErrorKind::NotTrait(self.trait_name));
        }

        let parameters = self
            .args_no_self
            .iter()
            .map(|a| Ok(a.lower(env)?))
            .collect::<Result<Vec<_>>>()?;

        if parameters.len() != k.binders.len() {
            bail!(
                "wrong number of parameters, expected `{:?}`, got `{:?}`",
                k.binders.len(),
                parameters.len()
            )
        }

        for (binder, param) in k.binders.binders.iter().zip(parameters.iter()) {
            check_type_kinds("incorrect kind for trait parameter", binder, param)?;
        }

        Ok(rust_ir::TraitBound {
            trait_id: id,
            args_no_self: parameters,
        })
    }
}

trait LowerProjectionEqBound {
    fn lower(&self, env: &Env) -> Result<rust_ir::ProjectionEqBound>;
}

impl LowerProjectionEqBound for ProjectionEqBound {
    fn lower(&self, env: &Env) -> Result<rust_ir::ProjectionEqBound> {
        let trait_bound = self.trait_bound.lower(env)?;
        let info = match env
            .associated_ty_infos
            .get(&(trait_bound.trait_id, self.name.str))
        {
            Some(info) => info,
            None => bail!("no associated type `{}` defined in trait", self.name.str),
        };
        let args: Vec<_> = self.args
            .iter()
            .map(|a| a.lower(env))
            .collect::<Result<_>>()?;

        if args.len() != info.addl_parameter_kinds.len() {
            bail!(
                "wrong number of parameters for associated type (expected {}, got {})",
                info.addl_parameter_kinds.len(),
                args.len()
            )
        }

        for (param, arg) in info.addl_parameter_kinds.iter().zip(args.iter()) {
            check_type_kinds("incorrect kind for associated type parameter", param, arg)?;
        }

        Ok(rust_ir::ProjectionEqBound {
            trait_bound,
            associated_ty_id: info.id,
            parameters: args,
            value: self.value.lower(env)?,
        })
    }
}

trait LowerInlineBound {
    fn lower(&self, env: &Env) -> Result<rust_ir::InlineBound>;
}

impl LowerInlineBound for InlineBound {
    fn lower(&self, env: &Env) -> Result<rust_ir::InlineBound> {
        let bound = match self {
            InlineBound::TraitBound(b) => rust_ir::InlineBound::TraitBound(b.lower(&env)?),
            InlineBound::ProjectionEqBound(b) => {
                rust_ir::InlineBound::ProjectionEqBound(b.lower(&env)?)
            }
        };
        Ok(bound)
    }
}

trait LowerQuantifiedInlineBound {
    fn lower(&self, env: &Env) -> Result<rust_ir::QuantifiedInlineBound>;
}

impl LowerQuantifiedInlineBound for QuantifiedInlineBound {
    fn lower(&self, env: &Env) -> Result<rust_ir::QuantifiedInlineBound> {
        let parameter_kinds = self.parameter_kinds.iter().map(|pk| pk.lower());
        let binders = env.in_binders(parameter_kinds, |env| Ok(self.bound.lower(env)?))?;
        Ok(binders)
    }
}

trait LowerQuantifiedInlineBoundVec {
    fn lower(&self, env: &Env) -> Result<Vec<rust_ir::QuantifiedInlineBound>>;
}

impl LowerQuantifiedInlineBoundVec for [QuantifiedInlineBound] {
    fn lower(&self, env: &Env) -> Result<Vec<rust_ir::QuantifiedInlineBound>> {
        self.iter().map(|b| b.lower(env)).collect()
    }
}

trait LowerPolarizedTraitRef {
    fn lower(&self, env: &Env) -> Result<rust_ir::PolarizedTraitRef>;
}

impl LowerPolarizedTraitRef for PolarizedTraitRef {
    fn lower(&self, env: &Env) -> Result<rust_ir::PolarizedTraitRef> {
        Ok(match *self {
            PolarizedTraitRef::Positive(ref tr) => {
                rust_ir::PolarizedTraitRef::Positive(tr.lower(env)?)
            }
            PolarizedTraitRef::Negative(ref tr) => {
                rust_ir::PolarizedTraitRef::Negative(tr.lower(env)?)
            }
        })
    }
}

trait LowerProjectionTy {
    fn lower(&self, env: &Env) -> Result<chalk_ir::ProjectionTy>;
}

impl LowerProjectionTy for ProjectionTy {
    fn lower(&self, env: &Env) -> Result<chalk_ir::ProjectionTy> {
        let ProjectionTy {
            ref trait_ref,
            ref name,
            ref args,
        } = *self;
        let chalk_ir::TraitRef {
            trait_id,
            parameters: trait_parameters,
        } = trait_ref.lower(env)?;
        let info = match env.associated_ty_infos.get(&(trait_id, name.str)) {
            Some(info) => info,
            None => bail!("no associated type `{}` defined in trait", name.str),
        };
        let mut args: Vec<_> = args
            .iter()
            .map(|a| a.lower(env))
            .collect::<Result<_>>()?;

        if args.len() != info.addl_parameter_kinds.len() {
            bail!(
                "wrong number of parameters for associated type (expected {}, got {})",
                info.addl_parameter_kinds.len(),
                args.len()
            )
        }

        for (param, arg) in info.addl_parameter_kinds.iter().zip(args.iter()) {
            check_type_kinds("incorrect kind for associated type parameter", param, arg)?;
        }

        args.extend(trait_parameters);

        Ok(chalk_ir::ProjectionTy {
            associated_ty_id: info.id,
            parameters: args,
        })
    }
}

trait LowerUnselectedProjectionTy {
    fn lower(&self, env: &Env) -> Result<chalk_ir::UnselectedProjectionTy>;
}

impl LowerUnselectedProjectionTy for UnselectedProjectionTy {
    fn lower(&self, env: &Env) -> Result<chalk_ir::UnselectedProjectionTy> {
        let parameters: Vec<_> = self.args
            .iter()
            .map(|a| a.lower(env))
            .collect::<Result<_>>()?;
        let ret = chalk_ir::UnselectedProjectionTy {
            type_name: self.name.str,
            parameters: parameters,
        };
        Ok(ret)
    }
}

trait LowerTy {
    fn lower(&self, env: &Env) -> Result<chalk_ir::Ty>;
}

impl LowerTy for Ty {
    fn lower(&self, env: &Env) -> Result<chalk_ir::Ty> {
        match *self {
            Ty::Id { name } => match env.lookup(name)? {
                NameLookup::Type(id) => {
                    let k = env.type_kind(id);
                    if k.binders.len() > 0 {
                        bail!(ErrorKind::IncorrectNumberOfTypeParameters(
                            name,
                            k.binders.len(),
                            0
                        ))
                    }

                    Ok(chalk_ir::Ty::Apply(chalk_ir::ApplicationTy {
                        name: chalk_ir::TypeName::ItemId(id),
                        parameters: vec![],
                    }))
                }
                NameLookup::Parameter(d) => Ok(chalk_ir::Ty::BoundVar(d)),
            },

            Ty::Apply { name, ref args } => {
                let id = match env.lookup(name)? {
                    NameLookup::Type(id) => id,
                    NameLookup::Parameter(_) => bail!(ErrorKind::CannotApplyTypeParameter(name)),
                };

                let k = env.type_kind(id);
                if k.binders.len() != args.len() {
                    bail!(ErrorKind::IncorrectNumberOfTypeParameters(
                        name,
                        k.binders.len(),
                        args.len()
                    ))
                }

                let parameters = args
                    .iter()
                    .map(|t| Ok(t.lower(env)?))
                    .collect::<Result<Vec<_>>>()?;

                for (param, arg) in k.binders.binders.iter().zip(args.iter()) {
                    check_type_kinds("incorrect parameter kind", param, arg)?;
                }

                Ok(chalk_ir::Ty::Apply(chalk_ir::ApplicationTy {
                    name: chalk_ir::TypeName::ItemId(id),
                    parameters: parameters,
                }))
            }

            Ty::Projection { ref proj } => Ok(chalk_ir::Ty::Projection(proj.lower(env)?)),

            Ty::UnselectedProjection { ref proj } => {
                Ok(chalk_ir::Ty::UnselectedProjection(proj.lower(env)?))
            }

            Ty::ForAll {
                ref lifetime_names,
                ref ty,
            } => {
                let quantified_env = env.introduce(
                    lifetime_names
                        .iter()
                        .map(|id| chalk_ir::ParameterKind::Lifetime(id.str)),
                )?;

                let ty = ty.lower(&quantified_env)?;
                let quantified_ty = chalk_ir::QuantifiedTy {
                    num_binders: lifetime_names.len(),
                    ty,
                };
                Ok(chalk_ir::Ty::ForAll(Box::new(quantified_ty)))
            }
        }
    }
}

trait LowerParameter {
    fn lower(&self, env: &Env) -> Result<chalk_ir::Parameter>;
}

impl LowerParameter for Parameter {
    fn lower(&self, env: &Env) -> Result<chalk_ir::Parameter> {
        match *self {
            Parameter::Ty(ref t) => Ok(chalk_ir::ParameterKind::Ty(t.lower(env)?)),
            Parameter::Lifetime(ref l) => Ok(chalk_ir::ParameterKind::Lifetime(l.lower(env)?)),
        }
    }
}

trait LowerLifetime {
    fn lower(&self, env: &Env) -> Result<chalk_ir::Lifetime>;
}

impl LowerLifetime for Lifetime {
    fn lower(&self, env: &Env) -> Result<chalk_ir::Lifetime> {
        match *self {
            Lifetime::Id { name } => match env.lookup_lifetime(name)? {
                LifetimeLookup::Parameter(d) => Ok(chalk_ir::Lifetime::BoundVar(d)),
            },
        }
    }
}

trait LowerImpl {
    fn lower_impl(&self, empty_env: &Env) -> Result<rust_ir::ImplDatum>;
}

impl LowerImpl for Impl {
    fn lower_impl(&self, empty_env: &Env) -> Result<rust_ir::ImplDatum> {
        let binders = empty_env.in_binders(self.all_parameters(), |env| {
            let trait_ref = self.trait_ref.lower(env)?;

            if !trait_ref.is_positive() && !self.assoc_ty_values.is_empty() {
                bail!("negative impls cannot define associated values");
            }

            let trait_id = trait_ref.trait_ref().trait_id;
            let where_clauses = self.lower_where_clauses(&env)?;
            let associated_ty_values = self.assoc_ty_values
                    .iter()
                    .map(|v| v.lower(trait_id, env))
                    .collect::<Result<_>>()?;
            Ok(rust_ir::ImplDatumBound {
                trait_ref,
                where_clauses,
                associated_ty_values,
                specialization_priority: 0,
                impl_type: match self.impl_type {
                    ImplType::Local => rust_ir::ImplType::Local,
                    ImplType::External => rust_ir::ImplType::External,
                },
            })
        })?;

        Ok(rust_ir::ImplDatum { binders: binders })
    }
}

trait LowerClause {
    fn lower_clause(&self, env: &Env) -> Result<Vec<chalk_ir::ProgramClause>>;
}

impl LowerClause for Clause {
    fn lower_clause(&self, env: &Env) -> Result<Vec<chalk_ir::ProgramClause>> {
        let implications = env.in_binders(self.all_parameters(), |env| {
            let consequences: Vec<chalk_ir::DomainGoal> = self.consequence.lower(env)?;

            let conditions: Vec<chalk_ir::Goal> = self
                .conditions
                .iter()
                .map(|g| g.lower(env).map(|g| *g))
                .rev() // (*)
                .collect::<Result<_>>()?;

            // (*) Subtle: in the SLG solver, we pop conditions from R to
            // L. To preserve the expected order (L to R), we must
            // therefore reverse.

            let implications = consequences
                .into_iter()
                .map(|consequence| chalk_ir::ProgramClauseImplication {
                    consequence,
                    conditions: conditions.clone(),
                }).collect::<Vec<_>>();
            Ok(implications)
        })?;

        let clauses = implications
            .into_iter()
            .map(
                |implication: chalk_ir::Binders<chalk_ir::ProgramClauseImplication>| {
                    if implication.binders.is_empty() {
                        chalk_ir::ProgramClause::Implies(implication.value)
                    } else {
                        chalk_ir::ProgramClause::ForAll(implication)
                    }
                },
            ).collect();
        Ok(clauses)
    }
}

trait LowerAssocTyValue {
    fn lower(&self, trait_id: chalk_ir::ItemId, env: &Env) -> Result<rust_ir::AssociatedTyValue>;
}

impl LowerAssocTyValue for AssocTyValue {
    fn lower(&self, trait_id: chalk_ir::ItemId, env: &Env) -> Result<rust_ir::AssociatedTyValue> {
        let info = &env.associated_ty_infos[&(trait_id, self.name.str)];
        let value = env.in_binders(self.all_parameters(), |env| {
            Ok(rust_ir::AssociatedTyValueBound {
                ty: self.value.lower(env)?,
            })
        })?;
        Ok(rust_ir::AssociatedTyValue {
            associated_ty_id: info.id,
            value: value,
        })
    }
}

trait LowerTrait {
    fn lower_trait(&self, trait_id: chalk_ir::ItemId, env: &Env) -> Result<rust_ir::TraitDatum>;
}

impl LowerTrait for TraitDefn {
    fn lower_trait(&self, trait_id: chalk_ir::ItemId, env: &Env) -> Result<rust_ir::TraitDatum> {
        let binders = env.in_binders(self.all_parameters(), |env| {
            let trait_ref = chalk_ir::TraitRef {
                trait_id: trait_id,
                parameters: self.parameter_refs(),
            };

            if self.flags.auto {
                if trait_ref.parameters.len() > 1 {
                    bail!("auto trait cannot have parameters");
                }
                if !self.where_clauses.is_empty() {
                    bail!("auto trait cannot have where clauses");
                }
            }

            Ok(rust_ir::TraitDatumBound {
                trait_ref: trait_ref,
                where_clauses: self.lower_where_clauses(env)?,
                flags: rust_ir::TraitFlags {
                    auto: self.flags.auto,
                    marker: self.flags.marker,
                    upstream: self.flags.upstream,
                    fundamental: self.flags.fundamental,
                    deref: self.flags.deref,
                },
            })
        })?;

        Ok(rust_ir::TraitDatum { binders: binders })
    }
}

pub trait LowerGoal<A> {
    fn lower(&self, arg: &A) -> Result<Box<chalk_ir::Goal>>;
}

impl LowerGoal<rust_ir::Program> for Goal {
    fn lower(&self, program: &rust_ir::Program) -> Result<Box<chalk_ir::Goal>> {
        let associated_ty_infos: BTreeMap<_, _> = program
            .associated_ty_data
            .iter()
            .map(|(&associated_ty_id, datum)| {
                let trait_datum = &program.trait_data[&datum.trait_id];
                let num_trait_params = trait_datum.binders.len();
                let num_addl_params = datum.parameter_kinds.len() - num_trait_params;
                let addl_parameter_kinds = datum.parameter_kinds[..num_addl_params].to_owned();
                let info = AssociatedTyInfo {
                    id: associated_ty_id,
                    addl_parameter_kinds,
                };
                ((datum.trait_id, datum.name), info)
            }).collect();

        let env = Env {
            type_ids: &program.type_ids,
            type_kinds: &program.type_kinds,
            associated_ty_infos: &associated_ty_infos,
            parameter_map: BTreeMap::new(),
        };

        self.lower(&env)
    }
}

impl<'k> LowerGoal<Env<'k>> for Goal {
    fn lower(&self, env: &Env<'k>) -> Result<Box<chalk_ir::Goal>> {
        match self {
            Goal::ForAll(ids, g) => g.lower_quantified(env, chalk_ir::QuantifierKind::ForAll, ids),
            Goal::Exists(ids, g) => g.lower_quantified(env, chalk_ir::QuantifierKind::Exists, ids),
            Goal::Implies(hyp, g) => {
                // We "elaborate" implied bounds by lowering goals like `T: Trait` and
                // `T: Trait<Assoc = U>` to `FromEnv(T: Trait)` and `FromEnv(T: Trait<Assoc = U>)`
                // in the assumptions of an `if` goal, e.g. `if (T: Trait) { ... }` lowers to
                // `if (FromEnv(T: Trait)) { ... /* this part is untouched */ ... }`.
                let where_clauses: Result<Vec<_>> = hyp
                    .into_iter()
                    .flat_map(|h| h.lower_clause(env).apply_result())
                    .map(|result| result.map(|h| h.into_from_env_clause()))
                    .collect();
                Ok(Box::new(chalk_ir::Goal::Implies(
                    where_clauses?,
                    g.lower(env)?,
                )))
            }
            Goal::And(g1, g2) => Ok(Box::new(chalk_ir::Goal::And(
                g1.lower(env)?,
                g2.lower(env)?,
            ))),
            Goal::Not(g) => Ok(Box::new(chalk_ir::Goal::Not(g.lower(env)?))),
            Goal::Compatible(g) => Ok(Box::new(g.lower(env)?.compatible())),
            Goal::Leaf(leaf) => {
                // A where clause can lower to multiple leaf goals; wrap these in Goal::And.
                let leaves = leaf.lower(env)?.into_iter().map(chalk_ir::Goal::Leaf);
                let goal = leaves
                    .fold1(|goal, leaf| chalk_ir::Goal::And(Box::new(goal), Box::new(leaf)))
                    .expect("at least one goal");
                Ok(Box::new(goal))
            }
        }
    }
}

trait LowerQuantifiedGoal {
    fn lower_quantified(
        &self,
        env: &Env,
        quantifier_kind: chalk_ir::QuantifierKind,
        parameter_kinds: &[ParameterKind],
    ) -> Result<Box<chalk_ir::Goal>>;
}

impl LowerQuantifiedGoal for Goal {
    fn lower_quantified(
        &self,
        env: &Env,
        quantifier_kind: chalk_ir::QuantifierKind,
        parameter_kinds: &[ParameterKind],
    ) -> Result<Box<chalk_ir::Goal>> {
        if parameter_kinds.is_empty() {
            return self.lower(env);
        }

        let parameter_kinds = parameter_kinds.iter().map(|pk| pk.lower());
        let subgoal = env.in_binders(parameter_kinds, |env| self.lower(env))?;
        Ok(Box::new(chalk_ir::Goal::Quantified(
            quantifier_kind,
            subgoal,
        )))
    }
}

/// Lowers Result<Vec<T>> -> Vec<Result<T>>.
trait ApplyResult {
    type Output;
    fn apply_result(self) -> Self::Output;
}

impl<T> ApplyResult for Result<Vec<T>> {
    type Output = Vec<Result<T>>;
    fn apply_result(self) -> Self::Output {
        match self {
            Ok(v) => v.into_iter().map(Ok).collect(),
            Err(e) => vec![Err(e)],
        }
    }
}

trait Kinded {
    fn kind(&self) -> Kind;
}

impl Kinded for ParameterKind {
    fn kind(&self) -> Kind {
        match *self {
            ParameterKind::Ty(_) => Kind::Ty,
            ParameterKind::Lifetime(_) => Kind::Lifetime,
        }
    }
}

impl Kinded for Parameter {
    fn kind(&self) -> Kind {
        match *self {
            Parameter::Ty(_) => Kind::Ty,
            Parameter::Lifetime(_) => Kind::Lifetime,
        }
    }
}

impl<T, L> Kinded for chalk_ir::ParameterKind<T, L> {
    fn kind(&self) -> Kind {
        match *self {
            chalk_ir::ParameterKind::Ty(_) => Kind::Ty,
            chalk_ir::ParameterKind::Lifetime(_) => Kind::Lifetime,
        }
    }
}
