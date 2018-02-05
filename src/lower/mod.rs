use std::collections::BTreeMap;

use chalk_parse::ast::*;
use lalrpop_intern::intern;

use cast::{Cast, Caster};
use errors::*;
use fold::shift::Shift;
use ir;
use solve::SolverChoice;

mod test;
mod default;
mod wf;

type TypeIds = BTreeMap<ir::Identifier, ir::ItemId>;
type TypeKinds = BTreeMap<ir::ItemId, ir::TypeKind>;
type AssociatedTyInfos = BTreeMap<(ir::ItemId, ir::Identifier), AssociatedTyInfo>;
type ParameterMap = BTreeMap<ir::ParameterKind<ir::Identifier>, usize>;

#[derive(Clone, Debug)]
struct Env<'k> {
    type_ids: &'k TypeIds,
    type_kinds: &'k TypeKinds,
    associated_ty_infos: &'k AssociatedTyInfos,
    parameter_map: ParameterMap,
}

#[derive(Debug, PartialEq, Eq)]
struct AssociatedTyInfo {
    id: ir::ItemId,
    addl_parameter_kinds: Vec<ir::ParameterKind<ir::Identifier>>,
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
        if let Some(k) = self.parameter_map
            .get(&ir::ParameterKind::Lifetime(name.str))
        {
            return Ok(LifetimeLookup::Parameter(*k));
        }

        bail!("invalid lifetime name: {:?}", name.str);
    }

    fn type_kind(&self, id: ir::ItemId) -> &ir::TypeKind {
        &self.type_kinds[&id]
    }

    /// Introduces new parameters, shifting the indices of existing
    /// parameters to accommodate them. The indices of the new binders
    /// will be assigned in order as they are iterated.
    fn introduce<I>(&self, binders: I) -> Self
    where
        I: IntoIterator<Item = ir::ParameterKind<ir::Identifier>>,
        I::IntoIter: ExactSizeIterator,
    {
        let binders = binders.into_iter().enumerate().map(|(i, k)| (k, i));
        let len = binders.len();
        let parameter_map: ParameterMap = self.parameter_map
            .iter()
            .map(|(&k, &v)| (k, v + len))
            .chain(binders)
            .collect();
        Env {
            parameter_map,
            ..*self
        }
    }

    fn in_binders<I, T, OP>(&self, binders: I, op: OP) -> Result<ir::Binders<T>>
    where
        I: IntoIterator<Item = ir::ParameterKind<ir::Identifier>>,
        I::IntoIter: ExactSizeIterator,
        OP: FnOnce(&Self) -> Result<T>,
    {
        let binders: Vec<_> = binders.into_iter().collect();
        let env = self.introduce(binders.iter().cloned());
        Ok(ir::Binders {
            binders: binders.anonymize(),
            value: op(&env)?,
        })
    }
}

pub trait LowerProgram {
    /// Lowers from a Program AST to the internal IR for a program.
    fn lower(&self, solver_choice: SolverChoice) -> Result<ir::Program>;

    /// As above, but skips the coherence step. This is a hack used
    /// internally in SLG testing to overcome shortcomings of the (for
    /// now...?)  default engine used in those checks.
    fn lower_without_coherence(&self) -> Result<ir::Program>;
}

impl LowerProgram for Program {
    fn lower(&self, solver_choice: SolverChoice) -> Result<ir::Program> {
        let mut program = self.lower_without_coherence()?;
        program.record_specialization_priorities(solver_choice)?;
        program.verify_well_formedness(solver_choice)?;
        Ok(program)
    }

    fn lower_without_coherence(&self) -> Result<ir::Program> {
        let mut index = 0;
        let mut next_item_id = || -> ir::ItemId {
            let i = index;
            index += 1;
            ir::ItemId { index: i }
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

                        associated_ty_data.insert(
                            info.id,
                            ir::AssociatedTyDatum {
                                trait_id: item_id,
                                id: info.id,
                                name: defn.name.str,
                                parameter_kinds: parameter_kinds,
                                where_clauses: vec![],
                            },
                        );
                    }
                }
                Item::Impl(ref d) => {
                    impl_data.insert(item_id, d.lower_impl(&empty_env)?);
                }
                Item::Clause(ref clause) => {
                    custom_clauses.push(clause.lower_clause(&empty_env)?);
                }
            }
        }

        let mut program = ir::Program {
            type_ids,
            type_kinds,
            struct_data,
            trait_data,
            impl_data,
            associated_ty_data,
            custom_clauses,
            default_impl_data: Vec::new(),
        };
        program.add_default_impls();
        Ok(program)
    }
}

trait LowerTypeKind {
    fn lower_type_kind(&self) -> Result<ir::TypeKind>;
}

trait LowerParameterMap {
    fn synthetic_parameters(&self) -> Option<ir::ParameterKind<ir::Identifier>>;
    fn declared_parameters(&self) -> &[ParameterKind];
    fn all_parameters(&self) -> Vec<ir::ParameterKind<ir::Identifier>> {
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

    fn parameter_refs(&self) -> Vec<ir::Parameter> {
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
            .enumerate()
            .map(|(index, id)| (id, index))
            .collect()
    }
}

impl LowerParameterMap for StructDefn {
    fn synthetic_parameters(&self) -> Option<ir::ParameterKind<ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for Impl {
    fn synthetic_parameters(&self) -> Option<ir::ParameterKind<ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for AssocTyDefn {
    fn synthetic_parameters(&self) -> Option<ir::ParameterKind<ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for AssocTyValue {
    fn synthetic_parameters(&self) -> Option<ir::ParameterKind<ir::Identifier>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
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

impl LowerParameterMap for Clause {
    fn synthetic_parameters(&self) -> Option<ir::ParameterKind<ir::Identifier>> {
        None
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

    fn lower_where_clauses(&self, env: &Env) -> Result<Vec<ir::DomainGoal>> {
        self.where_clauses().lower(env)
    }
}

impl LowerTypeKind for StructDefn {
    fn lower_type_kind(&self) -> Result<ir::TypeKind> {
        Ok(ir::TypeKind {
            sort: ir::TypeSort::Struct,
            name: self.name.str,
            binders: ir::Binders {
                binders: self.all_parameters().anonymize(),
                value: (),
            },
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
        let binders: Vec<_> = self.parameter_kinds.iter().map(|p| p.lower()).collect();
        Ok(ir::TypeKind {
            sort: ir::TypeSort::Trait,
            name: self.name.str,
            binders: ir::Binders {
                // for the purposes of the *type*, ignore `Self`:
                binders: binders.anonymize(),
                value: (),
            },
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
    fn lower(&self, env: &Env) -> Result<Vec<ir::DomainGoal>>;
}

impl LowerWhereClauseVec for [WhereClause] {
    fn lower(&self, env: &Env) -> Result<Vec<ir::DomainGoal>> {
        self.iter().map(|wc| wc.lower(env)).collect()
    }
}

trait LowerWhereClause<T> {
    fn lower(&self, env: &Env) -> Result<T>;
}

/// Lowers a where-clause in the context of a clause (i.e. in "negative"
/// position); this is limited to the kinds of where-clauses users can actually
/// type in Rust and well-formedness checks.
impl LowerWhereClause<ir::DomainGoal> for WhereClause {
    fn lower(&self, env: &Env) -> Result<ir::DomainGoal> {
        Ok(match *self {
            WhereClause::Implemented { ref trait_ref } => {
                ir::DomainGoal::Implemented(trait_ref.lower(env)?)
            }
            WhereClause::ProjectionEq {
                ref projection,
                ref ty,
            } => ir::DomainGoal::ProjectionEq(ir::ProjectionEq {
                projection: projection.lower(env)?,
                ty: ty.lower(env)?,
            }),
            WhereClause::Normalize {
                ref projection,
                ref ty,
            } => ir::DomainGoal::Normalize(ir::Normalize {
                projection: projection.lower(env)?,
                ty: ty.lower(env)?,
            }),
            WhereClause::TyWellFormed { ref ty } => ir::WellFormed::Ty(ty.lower(env)?).cast(),
            WhereClause::TraitRefWellFormed { ref trait_ref } => {
                ir::WellFormed::TraitRef(trait_ref.lower(env)?).cast()
            }
            WhereClause::TyFromEnv { ref ty } => ir::FromEnv::Ty(ty.lower(env)?).cast(),
            WhereClause::TraitRefFromEnv { ref trait_ref } => {
                ir::FromEnv::TraitRef(trait_ref.lower(env)?).cast()
            }
            WhereClause::UnifyTys { .. } | WhereClause::UnifyLifetimes { .. } => {
                bail!("this form of where-clause not allowed here")
            }
            WhereClause::TraitInScope { trait_name } => {
                let id = match env.lookup(trait_name)? {
                    NameLookup::Type(id) => id,
                    NameLookup::Parameter(_) => bail!(ErrorKind::NotTrait(trait_name)),
                };

                if env.type_kind(id).sort != ir::TypeSort::Trait {
                    bail!(ErrorKind::NotTrait(trait_name));
                }

                ir::DomainGoal::InScope(id)
            }
        })
    }
}

/// Lowers a where-clause in the context of a goal (i.e. in "positive"
/// position); this is richer in terms of the legal sorts of where-clauses that
/// can appear, because it includes all the sorts of things that the compiler
/// must verify.
impl LowerWhereClause<ir::LeafGoal> for WhereClause {
    fn lower(&self, env: &Env) -> Result<ir::LeafGoal> {
        Ok(match *self {
            WhereClause::Implemented { .. }
            | WhereClause::ProjectionEq { .. }
            | WhereClause::Normalize { .. } => {
                let g: ir::DomainGoal = self.lower(env)?;
                g.cast()
            }
            WhereClause::TyWellFormed { ref ty } => ir::WellFormed::Ty(ty.lower(env)?).cast(),
            WhereClause::TraitRefWellFormed { ref trait_ref } => {
                ir::WellFormed::TraitRef(trait_ref.lower(env)?).cast()
            }
            WhereClause::TyFromEnv { ref ty } => ir::FromEnv::Ty(ty.lower(env)?).cast(),
            WhereClause::TraitRefFromEnv { ref trait_ref } => {
                ir::FromEnv::TraitRef(trait_ref.lower(env)?).cast()
            }
            WhereClause::UnifyTys { ref a, ref b } => ir::EqGoal {
                a: ir::ParameterKind::Ty(a.lower(env)?),
                b: ir::ParameterKind::Ty(b.lower(env)?),
            }.cast(),
            WhereClause::UnifyLifetimes { ref a, ref b } => ir::EqGoal {
                a: ir::ParameterKind::Lifetime(a.lower(env)?),
                b: ir::ParameterKind::Lifetime(b.lower(env)?),
            }.cast(),
            WhereClause::TraitInScope { trait_name } => {
                let id = match env.lookup(trait_name)? {
                    NameLookup::Type(id) => id,
                    NameLookup::Parameter(_) => bail!(ErrorKind::NotTrait(trait_name)),
                };

                if env.type_kind(id).sort != ir::TypeSort::Trait {
                    bail!(ErrorKind::NotTrait(trait_name));
                }

                ir::DomainGoal::InScope(id).cast()
            }
        })
    }
}

trait LowerStructDefn {
    fn lower_struct(&self, item_id: ir::ItemId, env: &Env) -> Result<ir::StructDatum>;
}

impl LowerStructDefn for StructDefn {
    fn lower_struct(&self, item_id: ir::ItemId, env: &Env) -> Result<ir::StructDatum> {
        let binders = env.in_binders(self.all_parameters(), |env| {
            let self_ty = ir::ApplicationTy {
                name: ir::TypeName::ItemId(item_id),
                parameters: self.all_parameters()
                    .anonymize()
                    .iter()
                    .zip(0..)
                    .map(|p| p.to_parameter())
                    .collect(),
            };

            let fields: Result<_> = self.fields.iter().map(|f| f.ty.lower(env)).collect();
            let where_clauses = self.lower_where_clauses(env)?;

            Ok(ir::StructDatumBound {
                self_ty,
                fields: fields?,
                where_clauses,
            })
        })?;

        Ok(ir::StructDatum { binders })
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

        let parameters = self.args
            .iter()
            .map(|a| Ok(a.lower(env)?))
            .collect::<Result<Vec<_>>>()?;

        if parameters.len() != k.binders.len() + 1 {
            bail!(
                "wrong number of parameters, expected `{:?}`, got `{:?}`",
                k.binders.len() + 1,
                parameters.len()
            )
        }

        for (binder, param) in k.binders.binders.iter().zip(parameters.iter().skip(1)) {
            check_type_kinds("incorrect kind for trait parameter", binder, param)?;
        }

        Ok(ir::TraitRef {
            trait_id: id,
            parameters: parameters,
        })
    }
}

trait LowerPolarizedTraitRef {
    fn lower(&self, env: &Env) -> Result<ir::PolarizedTraitRef>;
}

impl LowerPolarizedTraitRef for PolarizedTraitRef {
    fn lower(&self, env: &Env) -> Result<ir::PolarizedTraitRef> {
        Ok(match *self {
            PolarizedTraitRef::Positive(ref tr) => ir::PolarizedTraitRef::Positive(tr.lower(env)?),
            PolarizedTraitRef::Negative(ref tr) => ir::PolarizedTraitRef::Negative(tr.lower(env)?),
        })
    }
}

trait LowerProjectionTy {
    fn lower(&self, env: &Env) -> Result<ir::ProjectionTy>;
}

impl LowerProjectionTy for ProjectionTy {
    fn lower(&self, env: &Env) -> Result<ir::ProjectionTy> {
        let ProjectionTy {
            ref trait_ref,
            ref name,
            ref args,
        } = *self;
        let ir::TraitRef {
            trait_id,
            parameters: trait_parameters,
        } = trait_ref.lower(env)?;
        let info = match env.associated_ty_infos.get(&(trait_id, name.str)) {
            Some(info) => info,
            None => bail!("no associated type `{}` defined in trait", name.str),
        };
        let mut args: Vec<_> = try!(args.iter().map(|a| a.lower(env)).collect());

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

        Ok(ir::ProjectionTy {
            associated_ty_id: info.id,
            parameters: args,
        })
    }
}

trait LowerUnselectedProjectionTy {
    fn lower(&self, env: &Env) -> Result<ir::UnselectedProjectionTy>;
}

impl LowerUnselectedProjectionTy for UnselectedProjectionTy {
    fn lower(&self, env: &Env) -> Result<ir::UnselectedProjectionTy> {
        let parameters: Vec<_> = try!(self.args.iter().map(|a| a.lower(env)).collect());
        let ret = ir::UnselectedProjectionTy {
            type_name: self.name.str,
            parameters: parameters,
        };
        Ok(ret)
    }
}

trait LowerTy {
    fn lower(&self, env: &Env) -> Result<ir::Ty>;
}

impl LowerTy for Ty {
    fn lower(&self, env: &Env) -> Result<ir::Ty> {
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

                    Ok(ir::Ty::Apply(ir::ApplicationTy {
                        name: ir::TypeName::ItemId(id),
                        parameters: vec![],
                    }))
                }
                NameLookup::Parameter(d) => Ok(ir::Ty::Var(d)),
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

                let parameters = args.iter()
                    .map(|t| Ok(t.lower(env)?))
                    .collect::<Result<Vec<_>>>()?;

                for (param, arg) in k.binders.binders.iter().zip(args.iter()) {
                    check_type_kinds("incorrect parameter kind", param, arg)?;
                }

                Ok(ir::Ty::Apply(ir::ApplicationTy {
                    name: ir::TypeName::ItemId(id),
                    parameters: parameters,
                }))
            }

            Ty::Projection { ref proj } => Ok(ir::Ty::Projection(proj.lower(env)?)),

            Ty::UnselectedProjection { ref proj } => {
                Ok(ir::Ty::UnselectedProjection(proj.lower(env)?))
            }

            Ty::ForAll {
                ref lifetime_names,
                ref ty,
            } => {
                let quantified_env = env.introduce(
                    lifetime_names
                        .iter()
                        .map(|id| ir::ParameterKind::Lifetime(id.str)),
                );

                let ty = ty.lower(&quantified_env)?;
                let quantified_ty = ir::QuantifiedTy {
                    num_binders: lifetime_names.len(),
                    ty,
                };
                Ok(ir::Ty::ForAll(Box::new(quantified_ty)))
            }
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
            Lifetime::Id { name } => match env.lookup_lifetime(name)? {
                LifetimeLookup::Parameter(d) => Ok(ir::Lifetime::Var(d)),
            },
        }
    }
}

trait LowerImpl {
    fn lower_impl(&self, empty_env: &Env) -> Result<ir::ImplDatum>;
}

impl LowerImpl for Impl {
    fn lower_impl(&self, empty_env: &Env) -> Result<ir::ImplDatum> {
        let binders = empty_env.in_binders(self.all_parameters(), |env| {
            let trait_ref = self.trait_ref.lower(env)?;

            if !trait_ref.is_positive() && !self.assoc_ty_values.is_empty() {
                bail!("negative impls cannot define associated values");
            }

            let trait_id = trait_ref.trait_ref().trait_id;
            let where_clauses = self.lower_where_clauses(&env)?;
            let associated_ty_values = try!(
                self.assoc_ty_values
                    .iter()
                    .map(|v| v.lower(trait_id, env))
                    .collect()
            );
            Ok(ir::ImplDatumBound {
                trait_ref,
                where_clauses,
                associated_ty_values,
                specialization_priority: 0,
            })
        })?;

        Ok(ir::ImplDatum { binders: binders })
    }
}

trait LowerClause {
    fn lower_clause(&self, empty_env: &Env) -> Result<ir::ProgramClause>;
}

impl LowerClause for Clause {
    fn lower_clause(&self, empty_env: &Env) -> Result<ir::ProgramClause> {
        let implication = empty_env.in_binders(self.all_parameters(), |env| {
            let consequence: ir::DomainGoal = self.consequence.lower(env)?;
            let mut conditions: Vec<ir::Goal> = self.conditions
                .iter()
                .map(|g| g.lower(env).map(|g| *g))
                .collect::<Result<_>>()?;

            // Subtle: in the SLG solver, we pop conditions from R to
            // L. To preserve the expected order (L to R), we must
            // therefore reverse.
            conditions.reverse();

            Ok(ir::ProgramClauseImplication {
                consequence,
                conditions,
            })
        })?;

        Ok(ir::ProgramClause {
            implication,
        })
    }
}

trait LowerAssocTyValue {
    fn lower(&self, trait_id: ir::ItemId, env: &Env) -> Result<ir::AssociatedTyValue>;
}

impl LowerAssocTyValue for AssocTyValue {
    fn lower(&self, trait_id: ir::ItemId, env: &Env) -> Result<ir::AssociatedTyValue> {
        let info = &env.associated_ty_infos[&(trait_id, self.name.str)];
        let value = env.in_binders(self.all_parameters(), |env| {
            Ok(ir::AssociatedTyValueBound {
                ty: self.value.lower(env)?,
                where_clauses: self.where_clauses.lower(env)?,
            })
        })?;
        Ok(ir::AssociatedTyValue {
            associated_ty_id: info.id,
            value: value,
        })
    }
}

trait LowerTrait {
    fn lower_trait(&self, trait_id: ir::ItemId, env: &Env) -> Result<ir::TraitDatum>;
}

impl LowerTrait for TraitDefn {
    fn lower_trait(&self, trait_id: ir::ItemId, env: &Env) -> Result<ir::TraitDatum> {
        let binders = env.in_binders(self.all_parameters(), |env| {
            let trait_ref = ir::TraitRef {
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

            Ok(ir::TraitDatumBound {
                trait_ref: trait_ref,
                where_clauses: self.lower_where_clauses(env)?,
                flags: ir::TraitFlags {
                    auto: self.flags.auto,
                    marker: self.flags.marker,
                },
            })
        })?;

        Ok(ir::TraitDatum { binders: binders })
    }
}

pub trait LowerGoal<A> {
    fn lower(&self, arg: &A) -> Result<Box<ir::Goal>>;
}

impl LowerGoal<ir::Program> for Goal {
    fn lower(&self, program: &ir::Program) -> Result<Box<ir::Goal>> {
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
            })
            .collect();

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
    fn lower(&self, env: &Env<'k>) -> Result<Box<ir::Goal>> {
        match *self {
            Goal::ForAll(ref ids, ref g) => {
                g.lower_quantified(env, ir::QuantifierKind::ForAll, ids)
            }
            Goal::Exists(ref ids, ref g) => {
                g.lower_quantified(env, ir::QuantifierKind::Exists, ids)
            }
            Goal::Implies(ref wc, ref g) => {
                Ok(Box::new(ir::Goal::Implies(wc.lower(env)?, g.lower(env)?)))
            }
            Goal::And(ref g1, ref g2) => {
                Ok(Box::new(ir::Goal::And(g1.lower(env)?, g2.lower(env)?)))
            }
            Goal::Not(ref g) => Ok(Box::new(ir::Goal::Not(g.lower(env)?))),
            Goal::Leaf(ref wc) => Ok(Box::new(ir::Goal::Leaf(wc.lower(env)?))),
        }
    }
}

trait LowerQuantifiedGoal {
    fn lower_quantified(
        &self,
        env: &Env,
        quantifier_kind: ir::QuantifierKind,
        parameter_kinds: &[ParameterKind],
    ) -> Result<Box<ir::Goal>>;
}

impl LowerQuantifiedGoal for Goal {
    fn lower_quantified(
        &self,
        env: &Env,
        quantifier_kind: ir::QuantifierKind,
        parameter_kinds: &[ParameterKind],
    ) -> Result<Box<ir::Goal>> {
        if parameter_kinds.is_empty() {
            return self.lower(env);
        }

        let parameter_kinds = parameter_kinds.iter().map(|pk| pk.lower());
        let subgoal = env.in_binders(parameter_kinds, |env| self.lower(env))?;
        Ok(Box::new(ir::Goal::Quantified(quantifier_kind, subgoal)))
    }
}

impl ir::Program {
    pub fn environment(&self) -> ir::ProgramEnvironment {
        // Construct the set of *clauses*; these are sort of a compiled form
        // of the data above that always has the form:
        //
        //       forall P0...Pn. Something :- Conditions
        let mut program_clauses = vec![];

        program_clauses.extend(self.custom_clauses.iter().cloned());

        program_clauses.extend(
            self.struct_data
                .values()
                .flat_map(|d| d.to_program_clauses()),
        );
        program_clauses.extend(
            self.trait_data
                .values()
                .flat_map(|d| d.to_program_clauses()),
        );
        program_clauses.extend(
            self.associated_ty_data
                .values()
                .flat_map(|d| d.to_program_clauses(self)),
        );
        program_clauses.extend(self.default_impl_data.iter().map(|d| d.to_program_clause()));

        for datum in self.impl_data.values() {
            // If we encounter a negative impl, do not generate any rule. Negative impls
            // are currently just there to deactivate default impls for auto traits.
            if datum.binders.value.trait_ref.is_positive() {
                program_clauses.push(datum.to_program_clause());
                program_clauses.extend(
                    datum
                        .binders
                        .value
                        .associated_ty_values
                        .iter()
                        .flat_map(|atv| atv.to_program_clauses(self, datum)),
                );
            }
        }

        let trait_data = self.trait_data.clone();
        let associated_ty_data = self.associated_ty_data.clone();

        ir::ProgramEnvironment {
            trait_data,
            associated_ty_data,
            program_clauses,
        }
    }
}

impl ir::ImplDatum {
    /// Given `impl<T: Clone> Clone for Vec<T>`, generate:
    ///
    /// ```notrust
    /// forall<T> { (Vec<T>: Clone) :- (T: Clone) }
    /// ```
    fn to_program_clause(&self) -> ir::ProgramClause {
        ir::ProgramClause {
            implication: self.binders.map_ref(|bound| {
                ir::ProgramClauseImplication {
                    consequence: bound.trait_ref.trait_ref().clone().cast(),
                    conditions: bound
                        .where_clauses
                        .iter()
                        .cloned()
                        .casted()
                        .collect(),
                }
            }),
        }
    }
}

impl ir::DefaultImplDatum {
    /// For each accessible type `T` in a struct which needs a default implementation for the auto
    /// trait `Foo` (accessible types are the struct fields types), we add a bound `T: Foo` (which
    /// is then expanded with `WF(T: Foo)`). For example, given:
    ///
    /// ```notrust
    /// #[auto] trait Send { }
    ///
    /// struct MyList<T> {
    ///     data: T,
    ///     next: Box<Option<MyList<T>>>,
    /// }
    ///
    /// ```
    ///
    /// generate:
    ///
    /// ```notrust
    /// forall<T> {
    ///     (MyList<T>: Send) :-
    ///         (T: Send),
    ///         (Box<Option<MyList<T>>>: Send)
    /// }
    /// ```
    fn to_program_clause(&self) -> ir::ProgramClause {
        ir::ProgramClause {
            implication: self.binders.map_ref(|bound| {
                ir::ProgramClauseImplication {
                    consequence: bound.trait_ref.clone().cast(),
                    conditions: {
                        let wc = bound.accessible_tys.iter().cloned().map(|ty| {
                            ir::TraitRef {
                                trait_id: bound.trait_ref.trait_id,
                                parameters: vec![ir::ParameterKind::Ty(ty)],
                            }
                        });

                        wc.casted().collect()
                    },
                }
            }),
        }
    }
}

impl ir::AssociatedTyValue {
    /// Given:
    ///
    /// ```notrust
    /// impl<T> Iterable for Vec<T> {
    ///     type IntoIter<'a> where T: 'a = Iter<'a, T>;
    /// }
    /// ```
    ///
    /// generate:
    ///
    /// ```notrust
    /// forall<'a, T> {
    ///     (Vec<T>: Iterable<IntoIter<'a> = Iter<'a, T>>) :-
    ///         (Vec<T>: Iterable),  // (1)
    ///         (T: 'a)              // (2)
    /// }
    /// ```
    ///
    /// and:
    ///
    /// ```notrust
    /// forall<'a, T> {
    ///     Vec<T>::IntoIter<'a> ==> Iter<'a, T> :-
    ///         InScope(Iterable),
    ///         <Vec<T> as Iterable>::IntoIter<'a> ==> Iter<'a, T>
    /// }
    /// ```
    fn to_program_clauses(
        &self,
        program: &ir::Program,
        impl_datum: &ir::ImplDatum,
    ) -> Vec<ir::ProgramClause> {
        // Begin with the innermost parameters (`'a`) and then add those from impl (`T`).
        let all_binders: Vec<_> = self.value
            .binders
            .iter()
            .cloned()
            .chain(impl_datum.binders.binders.iter().cloned())
            .collect();

        // Assemble the full list of conditions for projection to be valid.
        // This comes in two parts, marked as (1) and (2) in example above:
        //
        // 1. require that the trait is implemented
        // 2. any where-clauses from the `type` declaration in the impl
        let impl_trait_ref = impl_datum
            .binders
            .value
            .trait_ref
            .trait_ref()
            .up_shift(self.value.len());
        let conditions: Vec<ir::Goal> = Some(impl_trait_ref.clone().cast())
            .into_iter()
            .chain(self.value.value.where_clauses.clone().cast())
            .collect();

        let parameters: Vec<_> = {
            // First add refs to the bound parameters (`'a`, in above example)
            let parameters = self.value.binders.iter().zip(0..).map(|p| p.to_parameter());

            // Then add the trait-ref parameters (`Vec<T>`, in above example)
            parameters
                .chain(impl_trait_ref.parameters.clone())
                .collect()
        };

        let projection = ir::ProjectionTy {
            associated_ty_id: self.associated_ty_id,
            parameters: parameters.clone(),
        };

        let normalize_goal = ir::DomainGoal::Normalize(ir::Normalize {
            projection: projection.clone(),
            ty: self.value.value.ty.clone(),
        });

        // Determine the normalization
        let normalization = ir::ProgramClause {
            implication: ir::Binders {
                binders: all_binders.clone(),
                value: ir::ProgramClauseImplication {
                    consequence: normalize_goal.clone(),
                    conditions: conditions.clone(),
                },
            },
        };

        let unselected_projection = ir::UnselectedProjectionTy {
            type_name: program.associated_ty_data[&self.associated_ty_id]
                .name
                .clone(),
            parameters: parameters,
        };

        let unselected_normalization = ir::ProgramClause {
            implication: ir::Binders {
                binders: all_binders.clone(),
                value: ir::ProgramClauseImplication {
                    consequence: ir::DomainGoal::UnselectedNormalize(ir::UnselectedNormalize {
                        projection: unselected_projection,
                        ty: self.value.value.ty.clone(),
                    }),
                    conditions: vec![
                        normalize_goal.cast(),
                        ir::DomainGoal::InScope(impl_trait_ref.trait_id).cast(),
                    ],
                },
            },
        };

        vec![normalization, unselected_normalization]
    }
}

trait ToParameter {
    /// Utility for converting a list of all the binders into scope
    /// into references to those binders. Simply pair the binders with
    /// the indices, and invoke `to_parameter()` on the `(binder,
    /// index)` pair. The result will be a reference to a bound
    /// variable of appropriate kind at the corresponding index.
    fn to_parameter(&self) -> ir::Parameter;
}

impl<'a> ToParameter for (&'a ir::ParameterKind<()>, usize) {
    fn to_parameter(&self) -> ir::Parameter {
        let &(binder, index) = self;
        match *binder {
            ir::ParameterKind::Lifetime(_) => ir::ParameterKind::Lifetime(ir::Lifetime::Var(index)),
            ir::ParameterKind::Ty(_) => ir::ParameterKind::Ty(ir::Ty::Var(index)),
        }
    }
}

trait Anonymize {
    fn anonymize(&self) -> Vec<ir::ParameterKind<()>>;
}

impl Anonymize for [ir::ParameterKind<ir::Identifier>] {
    fn anonymize(&self) -> Vec<ir::ParameterKind<()>> {
        self.iter().map(|pk| pk.map(|_| ())).collect()
    }
}

impl ir::StructDatum {
    fn to_program_clauses(&self) -> Vec<ir::ProgramClause> {
        // Given:
        //
        //    struct Foo<T: Eq> { }
        //
        // we generate the following clause:
        //
        //    forall<T> { WF(Foo<T>) :- (T: Eq). }
        //    forall<T> { FromEnv(T: Eq) :- FromEnv(Foo<T>). }

        let wf = ir::ProgramClause {
            implication: self.binders.map_ref(|bound_datum| {
                ir::ProgramClauseImplication {
                    consequence: ir::WellFormed::Ty(bound_datum.self_ty.clone().cast()).cast(),

                    conditions: {
                        bound_datum.where_clauses
                                   .iter()
                                   .cloned()
                                   .casted()
                                   .collect()
                    },
                }
            }),
        };

        let mut clauses = vec![wf];
        let condition = ir::FromEnv::Ty(self.binders.value.self_ty.clone().cast());

        for wc in self.binders
                      .value
                      .where_clauses
                      .iter()
                      .cloned()
                      .map(|wc| wc.into_from_env_clause())
        {
            clauses.push(ir::ProgramClause {
                implication: self.binders.map_ref(|_| {
                    ir::ProgramClauseImplication {
                        consequence: wc.cast(),
                        conditions: vec![condition.clone().cast()],
                    }
                })
            });
        }

        clauses
    }
}

impl ir::TraitDatum {
    fn to_program_clauses(&self) -> Vec<ir::ProgramClause> {
        // Given:
        //
        //    trait Ord<T> where Self: Eq<T> { ... }
        //
        // we generate the following clause:
        //
        //    forall<Self, T> {
        //        WF(Self: Ord<T>) :- (Self: Ord<T>), WF(Self: Eq<T>)
        //    }
        //
        // and the reverse rules:
        //    forall<Self, T> { (Self: Ord<T>) :- FromEnv(Self: Ord<T>) }
        //    forall<Self, T> { FromEnv(Self: Ord<T>) :- FromEnv(Self: Ord<T>) }

        let trait_ref = self.binders.value.trait_ref.clone();

        let wf = ir::ProgramClause {
            implication: self.binders.map_ref(|bound| {
                ir::ProgramClauseImplication {
                    consequence: ir::WellFormed::TraitRef(trait_ref.clone()).cast(),

                    conditions: {
                        bound.where_clauses
                             .iter()
                             .cloned()
                             .map(|wc| wc.into_well_formed_clause().cast())
                             .chain(Some(ir::DomainGoal::Implemented(trait_ref.clone()).cast()))
                             .collect()
                    },
                }
            }),
        };

        let mut clauses = vec![wf];
        let condition = ir::FromEnv::TraitRef(trait_ref.clone());

        for wc in self.binders
                      .value
                      .where_clauses
                      .iter()
                      .cloned()
                      .map(|wc| wc.into_from_env_clause().cast())
                      .chain(Some(ir::DomainGoal::Implemented(trait_ref).cast()))
        {
            clauses.push(ir::ProgramClause {
                implication: self.binders.map_ref(|_| {
                    ir::ProgramClauseImplication {
                        consequence: wc,
                        conditions: vec![condition.clone().cast()],
                    }
                }),
            });
        }

        clauses
    }
}

impl ir::AssociatedTyDatum {
    fn to_program_clauses(&self, program: &ir::Program) -> Vec<ir::ProgramClause> {
        // For each associated type, we define the "projection
        // equality" rules. There are always two; one for a successful normalization,
        // and one for the "fallback" notion of equality.
        //
        // Given:
        //
        //    trait Foo {
        //        type Assoc;
        //    }
        //
        // we generate the 'fallback' rule:
        //
        //    forall<T> {
        //        ProjectionEq(<T as Foo>::Assoc = (Foo::Assoc)<T>) :-
        //            T: Foo
        //    }
        //
        // and
        //
        //    forall<T> {
        //        ProjectionEq(<T as Foo>::Assoc = U) :-
        //            Normalize(<T as Foo>::Assoc -> U)
        //    }
        //
        // We used to generate an "elaboration" rule like this:
        //
        //    forall<T> {
        //        T: Foo :-
        //            exists<U> { ProjectionEq(<T as Foo>::Assoc = U) }
        //    }
        //
        // but this caused problems with the recursive solver. In
        // particular, whenever normalization is possible, we cannot
        // solve that projection uniquely, since we can now elaborate
        // `ProjectionEq` to fallback *or* normalize it. So instead we
        // handle this kind of reasoning by expanding "projection
        // equality" predicates (see `DomainGoal::expanded`).
        //
        // We also generate rules specific to WF requirements and implied bounds,
        // see below.

        let binders: Vec<_> = self.parameter_kinds
            .iter()
            .map(|pk| pk.map(|_| ()))
            .collect();
        let parameters: Vec<_> = binders.iter().zip(0..).map(|p| p.to_parameter()).collect();
        let projection = ir::ProjectionTy {
            associated_ty_id: self.id,
            parameters: parameters.clone(),
        };

        // Retrieve the trait ref embedding the associated type
        let trait_ref = {
            let (associated_ty_data, trait_params, _) = program.split_projection(&projection);
            ir::TraitRef {
                trait_id: associated_ty_data.trait_id,
                parameters: trait_params.to_owned(),
            }
        };

        // Construct an application from the projection. So if we have `<T as Iterator>::Item`,
        // we would produce `(Iterator::Item)<T>`.
        let app = ir::ApplicationTy {
            name: ir::TypeName::AssociatedType(self.id),
            parameters,
        };
        let app_ty = ir::Ty::Apply(app);

        let mut clauses = vec![];

        //    forall<T> {
        //        ProjectionEq(<T as Foo>::Assoc = (Foo::Assoc)<T>) :-
        //            T: Foo
        //    }
        clauses.push(ir::ProgramClause {
            implication: ir::Binders {
                binders: binders.clone(),
                value: ir::ProgramClauseImplication {
                    consequence: ir::ProjectionEq {
                        projection: projection.clone(),
                        ty: app_ty.clone(),
                    }.cast(),
                    conditions: vec![trait_ref.clone().cast()],
                },
            },
        });

        // The above application type is always well-formed, and `<T as Foo>::Assoc` will
        // unify with `(Foo::Assoc)<T>` only if `T: Foo`, because of the above rule, so we have:
        //
        //    forall<T> {
        //        WellFormed((Foo::Assoc)<T>).
        //    }
        clauses.push(ir::ProgramClause {
            implication: ir::Binders {
                binders: binders.clone(),
                value: ir::ProgramClauseImplication {
                    consequence: ir::WellFormed::Ty(app_ty).cast(),
                    conditions: vec![],
                }
            }
        });

        // add new type parameter U
        let mut binders = binders;
        binders.push(ir::ParameterKind::Ty(()));
        let ty = ir::Ty::Var(binders.len() - 1);

        // `Normalize(<T as Foo>::Assoc -> U)`
        let normalize = ir::Normalize { projection: projection.clone(), ty: ty.clone() };

        //    forall<T> {
        //        ProjectionEq(<T as Foo>::Assoc = U) :-
        //            Normalize(<T as Foo>::Assoc -> U)
        //    }
        clauses.push(ir::ProgramClause {
            implication: ir::Binders {
                binders: binders.clone(),
                value: ir::ProgramClauseImplication {
                    consequence: ir::ProjectionEq {
                        projection: projection.clone(),
                        ty,
                    }.cast(),
                    conditions: vec![normalize.clone().cast()],
                },
            },
            });


        // We generate a proxy rule for the well-formedness of `T: Foo<Assoc = U>` which really
        // means two things: `T: Foo` and `Normalize(<T as Foo>::Assoc -> U)`. So we have the
        // following rule:
        //
        //    forall<T> {
        //        WellFormed(T: Foo<Assoc = U>) :-
        //            WellFormed(T: Foo), Normalize(<T as Foo>::Assoc -> U)
        //    }
        clauses.push(ir::ProgramClause {
            implication: ir::Binders {
                binders: binders.clone(),
                value: ir::ProgramClauseImplication {
                    consequence: ir::WellFormed::Normalize(normalize.clone()).cast(),
                    conditions: vec![
                        normalize.clone().cast(),
                        ir::WellFormed::TraitRef(trait_ref.clone()).cast()
                    ],
                }
            }
        });

        // We also have two proxy reverse rules, the first one being:
        //
        //    forall<T> {
        //        FromEnv(T: Foo) :- FromEnv(T: Foo<Assoc = U>)
        //    }
        clauses.push(ir::ProgramClause {
            implication: ir::Binders {
                binders: binders.clone(),
                value: ir::ProgramClauseImplication {
                    consequence: ir::FromEnv::TraitRef(trait_ref).cast(),
                    conditions: vec![ir::FromEnv::Normalize(normalize.clone()).cast()],
                },
            }
        });

        // And the other one being:
        //
        //    forall<T> {
        //        Normalize(<T as Foo>::Assoc -> U) :- FromEnv(T: Foo<Assoc = U>)
        //    }
        clauses.push(ir::ProgramClause {
            implication: ir::Binders {
                binders: binders,
                value: ir::ProgramClauseImplication {
                    consequence: normalize.clone().cast(),
                    conditions: vec![ir::FromEnv::Normalize(normalize).cast()],
                },
            }
        });

        clauses
    }
}
