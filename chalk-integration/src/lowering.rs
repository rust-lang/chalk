use chalk_ir::cast::{Cast, Caster};
use chalk_ir::interner::ChalkIr;
use chalk_ir::{self, AssocTypeId, ImplId, StructId, TraitId};
use chalk_parse::ast::*;
use chalk_rust_ir as rust_ir;
use chalk_rust_ir::{Anonymize, AssociatedTyValueId, IntoWhereClauses, ToParameter};
use lalrpop_intern::intern;
use std::collections::BTreeMap;
use std::sync::Arc;

use crate::error::RustIrError;
use crate::program::Program as LoweredProgram;
use crate::{Identifier as Ident, RawId, TypeKind, TypeSort};

type StructIds = BTreeMap<Ident, chalk_ir::StructId<ChalkIr>>;
type TraitIds = BTreeMap<Ident, chalk_ir::TraitId<ChalkIr>>;
type StructKinds = BTreeMap<chalk_ir::StructId<ChalkIr>, TypeKind>;
type TraitKinds = BTreeMap<chalk_ir::TraitId<ChalkIr>, TypeKind>;
type AssociatedTyLookups = BTreeMap<(chalk_ir::TraitId<ChalkIr>, Ident), AssociatedTyLookup>;
type AssociatedTyValueIds =
    BTreeMap<(chalk_ir::ImplId<ChalkIr>, Ident), AssociatedTyValueId<ChalkIr>>;
type ParameterMap = BTreeMap<chalk_ir::ParameterKind<Ident>, usize>;

pub type LowerResult<T> = Result<T, RustIrError>;

#[derive(Clone, Debug)]
struct Env<'k> {
    struct_ids: &'k StructIds,
    struct_kinds: &'k StructKinds,
    trait_ids: &'k TraitIds,
    trait_kinds: &'k TraitKinds,
    associated_ty_lookups: &'k AssociatedTyLookups,
    /// Parameter identifiers are used as keys, therefore
    /// all identifiers in an environment must be unique (no shadowing).
    parameter_map: ParameterMap,
}

impl<'k> Env<'k> {
    fn interner(&self) -> &ChalkIr {
        &ChalkIr
    }
}

/// Information about an associated type **declaration** (i.e., an
/// `AssociatedTyDatum`). This information is gathered in the first
/// phase of creating the Rust IR and is then later used to lookup the
/// "id" of an associated type.
///
/// ```ignore
/// trait Foo {
///     type Bar<'a>; // <-- associated type declaration
///          // ----
///          // |
///          // addl_parameter_kinds
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
struct AssociatedTyLookup {
    id: chalk_ir::AssocTypeId<ChalkIr>,
    addl_parameter_kinds: Vec<chalk_ir::ParameterKind<()>>,
}

enum TypeLookup {
    Struct(chalk_ir::StructId<ChalkIr>),
    Parameter(usize),
}

enum LifetimeLookup {
    Parameter(usize),
}

const SELF: &str = "Self";
const FIXME_SELF: &str = "__FIXME_SELF__";

impl<'k> Env<'k> {
    fn lookup_type(&self, name: Identifier) -> LowerResult<TypeLookup> {
        if let Some(k) = self
            .parameter_map
            .get(&chalk_ir::ParameterKind::Ty(name.str))
        {
            return Ok(TypeLookup::Parameter(*k));
        }

        if let Some(id) = self.struct_ids.get(&name.str) {
            return Ok(TypeLookup::Struct(*id));
        }

        if let Some(_) = self.trait_ids.get(&name.str) {
            return Err(RustIrError::NotStruct(name));
        }

        Err(RustIrError::InvalidTypeName(name))
    }

    fn lookup_trait(&self, name: Identifier) -> LowerResult<TraitId<ChalkIr>> {
        if let Some(_) = self
            .parameter_map
            .get(&chalk_ir::ParameterKind::Ty(name.str))
        {
            return Err(RustIrError::NotTrait(name));
        }

        if let Some(_) = self.struct_ids.get(&name.str) {
            return Err(RustIrError::NotTrait(name));
        }

        if let Some(id) = self.trait_ids.get(&name.str) {
            return Ok(*id);
        }

        Err(RustIrError::InvalidTypeName(name))
    }

    fn lookup_lifetime(&self, name: Identifier) -> LowerResult<LifetimeLookup> {
        if let Some(k) = self
            .parameter_map
            .get(&chalk_ir::ParameterKind::Lifetime(name.str))
        {
            return Ok(LifetimeLookup::Parameter(*k));
        }

        Err(RustIrError::InvalidLifetimeName(name))
    }

    fn struct_kind(&self, id: chalk_ir::StructId<ChalkIr>) -> &TypeKind {
        &self.struct_kinds[&id]
    }

    fn trait_kind(&self, id: chalk_ir::TraitId<ChalkIr>) -> &TypeKind {
        &self.trait_kinds[&id]
    }

    /// Introduces new parameters, shifting the indices of existing
    /// parameters to accommodate them. The indices of the new binders
    /// will be assigned in order as they are iterated.
    fn introduce<I>(&self, binders: I) -> LowerResult<Self>
    where
        I: IntoIterator<Item = chalk_ir::ParameterKind<Ident>>,
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
            Err(RustIrError::DuplicateOrShadowedParameters)?;
        }
        Ok(Env {
            parameter_map,
            ..*self
        })
    }

    fn in_binders<I, T, OP>(&self, binders: I, op: OP) -> LowerResult<chalk_ir::Binders<T>>
    where
        I: IntoIterator<Item = chalk_ir::ParameterKind<Ident>>,
        I::IntoIter: ExactSizeIterator,
        OP: FnOnce(&Self) -> LowerResult<T>,
    {
        let binders: Vec<_> = binders.into_iter().collect();
        let env = self.introduce(binders.iter().cloned())?;
        Ok(chalk_ir::Binders {
            binders: binders.anonymize(),
            value: op(&env)?,
        })
    }
}

pub(crate) trait LowerProgram {
    /// Lowers from a Program AST to the internal IR for a program.
    fn lower(&self) -> LowerResult<LoweredProgram>;
}

impl LowerProgram for Program {
    fn lower(&self) -> LowerResult<LoweredProgram> {
        let mut index = 0;
        let mut next_item_id = || -> RawId {
            let i = index;
            index += 1;
            RawId { index: i }
        };

        // Make a vector mapping each thing in `items` to an id,
        // based just on its position:
        let raw_ids: Vec<_> = self.items.iter().map(|_| next_item_id()).collect();

        // Create ids for associated type declarations and values
        let mut associated_ty_lookups = BTreeMap::new();
        let mut associated_ty_value_ids = BTreeMap::new();
        for (item, &raw_id) in self.items.iter().zip(&raw_ids) {
            match item {
                Item::TraitDefn(d) => {
                    if d.flags.auto && !d.assoc_ty_defns.is_empty() {
                        Err(RustIrError::AutoTraitAssociatedTypes(d.name))?;
                    }
                    for defn in &d.assoc_ty_defns {
                        let addl_parameter_kinds = defn.all_parameters();
                        let lookup = AssociatedTyLookup {
                            id: AssocTypeId(next_item_id()),
                            addl_parameter_kinds: addl_parameter_kinds.anonymize(),
                        };
                        associated_ty_lookups.insert((TraitId(raw_id), defn.name.str), lookup);
                    }
                }

                Item::Impl(d) => {
                    for atv in &d.assoc_ty_values {
                        let atv_id = AssociatedTyValueId(next_item_id());
                        associated_ty_value_ids.insert((ImplId(raw_id), atv.name.str), atv_id);
                    }
                }

                _ => {}
            }
        }

        let mut struct_ids = BTreeMap::new();
        let mut trait_ids = BTreeMap::new();
        let mut struct_kinds = BTreeMap::new();
        let mut trait_kinds = BTreeMap::new();
        for (item, &raw_id) in self.items.iter().zip(&raw_ids) {
            match item {
                Item::StructDefn(defn) => {
                    let type_kind = defn.lower_type_kind()?;
                    let id = StructId(raw_id);
                    struct_ids.insert(type_kind.name, id);
                    struct_kinds.insert(id, type_kind);
                }
                Item::TraitDefn(defn) => {
                    let type_kind = defn.lower_type_kind()?;
                    let id = TraitId(raw_id);
                    trait_ids.insert(type_kind.name, id);
                    trait_kinds.insert(id, type_kind);
                }
                Item::Impl(_) => continue,
                Item::Clause(_) => continue,
            };
        }

        let mut struct_data = BTreeMap::new();
        let mut trait_data = BTreeMap::new();
        let mut impl_data = BTreeMap::new();
        let mut associated_ty_data = BTreeMap::new();
        let mut associated_ty_values = BTreeMap::new();
        let mut custom_clauses = Vec::new();
        for (item, &raw_id) in self.items.iter().zip(&raw_ids) {
            let empty_env = Env {
                struct_ids: &struct_ids,
                struct_kinds: &struct_kinds,
                trait_ids: &trait_ids,
                trait_kinds: &trait_kinds,
                associated_ty_lookups: &associated_ty_lookups,
                parameter_map: BTreeMap::new(),
            };

            match *item {
                Item::StructDefn(ref d) => {
                    let struct_id = StructId(raw_id);
                    struct_data.insert(struct_id, Arc::new(d.lower_struct(struct_id, &empty_env)?));
                }
                Item::TraitDefn(ref trait_defn) => {
                    let trait_id = TraitId(raw_id);
                    trait_data.insert(
                        trait_id,
                        Arc::new(trait_defn.lower_trait(trait_id, &empty_env)?),
                    );

                    for assoc_ty_defn in &trait_defn.assoc_ty_defns {
                        let lookup = &associated_ty_lookups[&(trait_id, assoc_ty_defn.name.str)];

                        // The parameters in scope for the associated
                        // type definitions are *both* those from the
                        // trait *and* those from the associated type
                        // itself.
                        //
                        // Insert the associated type parameters first
                        // into the list so that they are given the
                        // indices starting from 0. This corresponds
                        // to the "de bruijn" convention where "more
                        // inner" sets of parameters get the lower
                        // indices:
                        //
                        // e.g., in this example, the indices would be
                        // assigned `[A0, A1, T0, T1]`:
                        //
                        // ```
                        // trait Foo<T0, T1> {
                        //     type Bar<A0, A1>;
                        // }
                        // ```
                        let mut parameter_kinds = assoc_ty_defn.all_parameters();
                        parameter_kinds.extend(trait_defn.all_parameters());

                        let binders = empty_env.in_binders(parameter_kinds, |env| {
                            Ok(rust_ir::AssociatedTyDatumBound {
                                bounds: assoc_ty_defn.bounds.lower(&env)?,
                                where_clauses: assoc_ty_defn.where_clauses.lower(&env)?,
                            })
                        })?;

                        associated_ty_data.insert(
                            lookup.id,
                            Arc::new(rust_ir::AssociatedTyDatum {
                                trait_id: TraitId(raw_id),
                                id: lookup.id,
                                name: assoc_ty_defn.name.str,
                                binders: binders,
                            }),
                        );
                    }
                }
                Item::Impl(ref impl_defn) => {
                    let impl_id = ImplId(raw_id);
                    let impl_datum = Arc::new(impl_defn.lower_impl(
                        &empty_env,
                        impl_id,
                        &associated_ty_value_ids,
                    )?);
                    impl_data.insert(impl_id, impl_datum.clone());
                    let trait_id = impl_datum.trait_id();

                    for atv in &impl_defn.assoc_ty_values {
                        let atv_id = associated_ty_value_ids[&(impl_id, atv.name.str)];
                        let lookup = &associated_ty_lookups[&(trait_id, atv.name.str)];

                        // The parameters in scope for the associated
                        // type definitions are *both* those from the
                        // impl *and* those from the associated type
                        // itself. As in the "trait" case above, we begin
                        // with the parameters from the impl.
                        let mut parameter_kinds = atv.all_parameters();
                        parameter_kinds.extend(impl_defn.all_parameters());

                        let value = empty_env.in_binders(parameter_kinds, |env| {
                            Ok(rust_ir::AssociatedTyValueBound {
                                ty: atv.value.lower(env)?,
                            })
                        })?;

                        associated_ty_values.insert(
                            atv_id,
                            Arc::new(rust_ir::AssociatedTyValue {
                                impl_id,
                                associated_ty_id: lookup.id,
                                value,
                            }),
                        );
                    }
                }
                Item::Clause(ref clause) => {
                    custom_clauses.extend(clause.lower_clause(&empty_env)?);
                }
            }
        }

        let program = LoweredProgram {
            struct_ids,
            trait_ids,
            struct_kinds,
            trait_kinds,
            struct_data,
            trait_data,
            impl_data,
            associated_ty_values,
            associated_ty_data,
            custom_clauses,
        };

        Ok(program)
    }
}

trait LowerTypeKind {
    fn lower_type_kind(&self) -> LowerResult<TypeKind>;
}

trait LowerParameterMap {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<Ident>>;
    fn declared_parameters(&self) -> &[ParameterKind];
    fn all_parameters(&self) -> Vec<chalk_ir::ParameterKind<Ident>> {
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

    fn parameter_refs(&self) -> Vec<chalk_ir::Parameter<ChalkIr>> {
        self.all_parameters()
            .anonymize()
            .iter()
            .zip(0..)
            .map(|p| p.to_parameter(self.interner()))
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
        self.all_parameters().into_iter().zip(0..).collect()
    }

    fn interner(&self) -> &ChalkIr {
        &ChalkIr
    }
}

impl LowerParameterMap for StructDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<Ident>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for Impl {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<Ident>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for AssocTyDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<Ident>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for AssocTyValue {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<Ident>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for TraitDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<Ident>> {
        Some(chalk_ir::ParameterKind::Ty(intern(SELF)))
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

impl LowerParameterMap for Clause {
    fn synthetic_parameters(&self) -> Option<chalk_ir::ParameterKind<Ident>> {
        None
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        &self.parameter_kinds
    }
}

trait LowerParameterKind {
    fn lower(&self) -> chalk_ir::ParameterKind<Ident>;
}

impl LowerParameterKind for ParameterKind {
    fn lower(&self) -> chalk_ir::ParameterKind<Ident> {
        match *self {
            ParameterKind::Ty(ref n) => chalk_ir::ParameterKind::Ty(n.str),
            ParameterKind::Lifetime(ref n) => chalk_ir::ParameterKind::Lifetime(n.str),
        }
    }
}

trait LowerWhereClauses {
    fn where_clauses(&self) -> &[QuantifiedWhereClause];

    fn lower_where_clauses(
        &self,
        env: &Env,
    ) -> LowerResult<Vec<chalk_ir::QuantifiedWhereClause<ChalkIr>>> {
        self.where_clauses().lower(env)
    }
}

impl LowerTypeKind for StructDefn {
    fn lower_type_kind(&self) -> LowerResult<TypeKind> {
        Ok(TypeKind {
            sort: TypeSort::Struct,
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
    fn lower_type_kind(&self) -> LowerResult<TypeKind> {
        let binders: Vec<_> = self.parameter_kinds.iter().map(|p| p.lower()).collect();
        Ok(TypeKind {
            sort: TypeSort::Trait,
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
    fn lower(&self, env: &Env) -> LowerResult<Vec<chalk_ir::QuantifiedWhereClause<ChalkIr>>>;
}

impl LowerWhereClauseVec for [QuantifiedWhereClause] {
    fn lower(&self, env: &Env) -> LowerResult<Vec<chalk_ir::QuantifiedWhereClause<ChalkIr>>> {
        self.iter()
            .flat_map(|wc| match wc.lower(env) {
                Ok(v) => v.into_iter().map(Ok).collect(),
                Err(e) => vec![Err(e)],
            })
            .collect()
    }
}

trait LowerWhereClause<T> {
    /// Lower from an AST `where` clause to an internal IR.
    /// Some AST `where` clauses can lower to multiple ones, this is why we return a `Vec`.
    /// As for now, this is the only the case for `where T: Foo<Item = U>` which lowers to
    /// `Implemented(T: Foo)` and `AliasEq(<T as Foo>::Item = U)`.
    fn lower(&self, env: &Env) -> LowerResult<Vec<T>>;
}

impl LowerWhereClause<chalk_ir::WhereClause<ChalkIr>> for WhereClause {
    fn lower(&self, env: &Env) -> LowerResult<Vec<chalk_ir::WhereClause<ChalkIr>>> {
        let where_clauses = match self {
            WhereClause::Implemented { trait_ref } => {
                vec![chalk_ir::WhereClause::Implemented(trait_ref.lower(env)?)]
            }
            WhereClause::AliasEq { alias, ty } => vec![
                chalk_ir::WhereClause::AliasEq(chalk_ir::AliasEq {
                    alias: alias.lower(env)?,
                    ty: ty.lower(env)?,
                }),
                chalk_ir::WhereClause::Implemented(alias.trait_ref.lower(env)?),
            ],
        };
        Ok(where_clauses)
    }
}
impl LowerWhereClause<chalk_ir::QuantifiedWhereClause<ChalkIr>> for QuantifiedWhereClause {
    fn lower(&self, env: &Env) -> LowerResult<Vec<chalk_ir::QuantifiedWhereClause<ChalkIr>>> {
        let parameter_kinds = self.parameter_kinds.iter().map(|pk| pk.lower());
        let binders = env.in_binders(parameter_kinds, |env| Ok(self.where_clause.lower(env)?))?;
        Ok(binders.into_iter().collect())
    }
}

trait LowerDomainGoal {
    fn lower(&self, env: &Env) -> LowerResult<Vec<chalk_ir::DomainGoal<ChalkIr>>>;
}

impl LowerDomainGoal for DomainGoal {
    fn lower(&self, env: &Env) -> LowerResult<Vec<chalk_ir::DomainGoal<ChalkIr>>> {
        let interner = env.interner();
        let goals = match self {
            DomainGoal::Holds { where_clause } => where_clause
                .lower(env)?
                .into_iter()
                .casted(interner)
                .collect(),
            DomainGoal::Normalize { alias, ty } => {
                vec![chalk_ir::DomainGoal::Normalize(chalk_ir::Normalize {
                    alias: alias.lower(env)?,
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
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Goal<ChalkIr>>;
}

impl LowerLeafGoal for LeafGoal {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Goal<ChalkIr>> {
        let interner = env.interner();
        Ok(match self {
            LeafGoal::DomainGoal { goal } => {
                chalk_ir::Goal::all(interner, goal.lower(env)?.into_iter().casted(interner))
            }
            LeafGoal::UnifyTys { a, b } => chalk_ir::EqGoal {
                a: a.lower(env)?.cast(interner),
                b: b.lower(env)?.cast(interner),
            }
            .cast::<chalk_ir::Goal<ChalkIr>>(interner),
            LeafGoal::UnifyLifetimes { ref a, ref b } => chalk_ir::EqGoal {
                a: a.lower(env)?.cast(interner),
                b: b.lower(env)?.cast(interner),
            }
            .cast::<chalk_ir::Goal<ChalkIr>>(interner),
        })
    }
}

trait LowerStructDefn {
    fn lower_struct(
        &self,
        struct_id: chalk_ir::StructId<ChalkIr>,
        env: &Env,
    ) -> LowerResult<rust_ir::StructDatum<ChalkIr>>;
}

impl LowerStructDefn for StructDefn {
    fn lower_struct(
        &self,
        struct_id: chalk_ir::StructId<ChalkIr>,
        env: &Env,
    ) -> LowerResult<rust_ir::StructDatum<ChalkIr>> {
        if self.flags.fundamental && self.all_parameters().len() != 1 {
            Err(RustIrError::InvalidFundamentalTypesParameters(self.name))?;
        }

        let binders = env.in_binders(self.all_parameters(), |env| {
            let fields: LowerResult<_> = self.fields.iter().map(|f| f.ty.lower(env)).collect();
            let where_clauses = self.lower_where_clauses(env)?;

            Ok(rust_ir::StructDatumBound {
                fields: fields?,
                where_clauses,
            })
        })?;

        let flags = rust_ir::StructFlags {
            upstream: self.flags.upstream,
            fundamental: self.flags.fundamental,
        };

        Ok(rust_ir::StructDatum {
            id: struct_id,
            binders,
            flags,
        })
    }
}

trait LowerTraitRef {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::TraitRef<ChalkIr>>;
}

impl LowerTraitRef for TraitRef {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::TraitRef<ChalkIr>> {
        let interner = env.interner();
        let without_self = TraitBound {
            trait_name: self.trait_name,
            args_no_self: self.args.iter().cloned().skip(1).collect(),
        }
        .lower(env)?;

        let self_parameter = self.args[0].lower(env)?;
        Ok(without_self.as_trait_ref(interner, self_parameter.assert_ty_ref(interner).clone()))
    }
}

trait LowerTraitBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::TraitBound<ChalkIr>>;
}

impl LowerTraitBound for TraitBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::TraitBound<ChalkIr>> {
        let trait_id = env.lookup_trait(self.trait_name)?;

        let k = env.trait_kind(trait_id);
        if k.sort != TypeSort::Trait {
            Err(RustIrError::NotTrait(self.trait_name))?;
        }

        let parameters = self
            .args_no_self
            .iter()
            .map(|a| Ok(a.lower(env)?))
            .collect::<LowerResult<Vec<_>>>()?;

        if parameters.len() != k.binders.len() {
            Err(RustIrError::IncorrectNumberOfTypeParameters {
                identifier: self.trait_name,
                expected: k.binders.len(),
                actual: parameters.len(),
            })?;
        }

        for (binder, param) in k.binders.binders.iter().zip(parameters.iter()) {
            if binder.kind() != param.kind() {
                Err(RustIrError::IncorrectTraitParameterKind {
                    identifier: self.trait_name,
                    expected: binder.kind(),
                    actual: param.kind(),
                })?;
            }
        }

        Ok(rust_ir::TraitBound {
            trait_id,
            args_no_self: parameters,
        })
    }
}

trait LowerAliasEqBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::AliasEqBound<ChalkIr>>;
}

impl LowerAliasEqBound for AliasEqBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::AliasEqBound<ChalkIr>> {
        let trait_bound = self.trait_bound.lower(env)?;
        let lookup = match env
            .associated_ty_lookups
            .get(&(trait_bound.trait_id, self.name.str))
        {
            Some(lookup) => lookup,
            None => Err(RustIrError::MissingAssociatedType(self.name))?,
        };
        let args: Vec<_> = self
            .args
            .iter()
            .map(|a| a.lower(env))
            .collect::<LowerResult<_>>()?;

        if args.len() != lookup.addl_parameter_kinds.len() {
            Err(RustIrError::IncorrectNumberOfAssociatedTypeParameters {
                identifier: self.name,
                expected: lookup.addl_parameter_kinds.len(),
                actual: args.len(),
            })?;
        }

        for (param, arg) in lookup.addl_parameter_kinds.iter().zip(args.iter()) {
            if param.kind() != arg.kind() {
                Err(RustIrError::IncorrectAssociatedTypeParameterKind {
                    identifier: self.name,
                    expected: param.kind(),
                    actual: arg.kind(),
                })?;
            }
        }

        Ok(rust_ir::AliasEqBound {
            trait_bound,
            associated_ty_id: lookup.id,
            parameters: args,
            value: self.value.lower(env)?,
        })
    }
}

trait LowerInlineBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::InlineBound<ChalkIr>>;
}

impl LowerInlineBound for InlineBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::InlineBound<ChalkIr>> {
        let bound = match self {
            InlineBound::TraitBound(b) => rust_ir::InlineBound::TraitBound(b.lower(&env)?),
            InlineBound::AliasEqBound(b) => rust_ir::InlineBound::AliasEqBound(b.lower(&env)?),
        };
        Ok(bound)
    }
}

trait LowerQuantifiedInlineBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::QuantifiedInlineBound<ChalkIr>>;
}

impl LowerQuantifiedInlineBound for QuantifiedInlineBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::QuantifiedInlineBound<ChalkIr>> {
        let parameter_kinds = self.parameter_kinds.iter().map(|pk| pk.lower());
        let binders = env.in_binders(parameter_kinds, |env| Ok(self.bound.lower(env)?))?;
        Ok(binders)
    }
}

trait LowerQuantifiedInlineBoundVec {
    fn lower(&self, env: &Env) -> LowerResult<Vec<rust_ir::QuantifiedInlineBound<ChalkIr>>>;
}

impl LowerQuantifiedInlineBoundVec for [QuantifiedInlineBound] {
    fn lower(&self, env: &Env) -> LowerResult<Vec<rust_ir::QuantifiedInlineBound<ChalkIr>>> {
        self.iter().map(|b| b.lower(env)).collect()
    }
}

trait LowerPolarity {
    fn lower(&self) -> rust_ir::Polarity;
}

impl LowerPolarity for Polarity {
    fn lower(&self) -> rust_ir::Polarity {
        match *self {
            Polarity::Positive => rust_ir::Polarity::Positive,
            Polarity::Negative => rust_ir::Polarity::Negative,
        }
    }
}

trait LowerImplType {
    fn lower(&self) -> rust_ir::ImplType;
}

impl LowerImplType for ImplType {
    fn lower(&self) -> rust_ir::ImplType {
        match self {
            ImplType::Local => rust_ir::ImplType::Local,
            ImplType::External => rust_ir::ImplType::External,
        }
    }
}

trait LowerTraitFlags {
    fn lower(&self) -> rust_ir::TraitFlags;
}

impl LowerTraitFlags for TraitFlags {
    fn lower(&self) -> rust_ir::TraitFlags {
        rust_ir::TraitFlags {
            auto: self.auto,
            marker: self.marker,
            upstream: self.upstream,
            fundamental: self.fundamental,
            non_enumerable: self.non_enumerable,
            coinductive: self.coinductive,
        }
    }
}

trait LowerAliasTy {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::AliasTy<ChalkIr>>;
}

impl LowerAliasTy for AliasTy {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::AliasTy<ChalkIr>> {
        let AliasTy {
            ref trait_ref,
            ref name,
            ref args,
        } = *self;
        let interner = env.interner();
        let chalk_ir::TraitRef {
            trait_id,
            substitution: trait_substitution,
        } = trait_ref.lower(env)?;
        let lookup = match env.associated_ty_lookups.get(&(trait_id.into(), name.str)) {
            Some(lookup) => lookup,
            None => Err(RustIrError::MissingAssociatedType(self.name))?,
        };
        let mut args: Vec<_> = args
            .iter()
            .map(|a| a.lower(env))
            .collect::<LowerResult<_>>()?;

        if args.len() != lookup.addl_parameter_kinds.len() {
            Err(RustIrError::IncorrectNumberOfAssociatedTypeParameters {
                identifier: self.name,
                expected: lookup.addl_parameter_kinds.len(),
                actual: args.len(),
            })?;
        }

        for (param, arg) in lookup.addl_parameter_kinds.iter().zip(args.iter()) {
            if param.kind() != arg.kind() {
                Err(RustIrError::IncorrectAssociatedTypeParameterKind {
                    identifier: self.name,
                    expected: param.kind(),
                    actual: arg.kind(),
                })?;
            }
        }

        args.extend(trait_substitution.iter(interner).cloned());

        Ok(chalk_ir::AliasTy {
            associated_ty_id: lookup.id,
            substitution: chalk_ir::Substitution::from(interner, args),
        })
    }
}

trait LowerTy {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Ty<ChalkIr>>;
}

impl LowerTy for Ty {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Ty<ChalkIr>> {
        let interner = env.interner();
        match *self {
            Ty::Id { name } => match env.lookup_type(name)? {
                TypeLookup::Struct(id) => {
                    let k = env.struct_kind(id);
                    if k.binders.len() > 0 {
                        Err(RustIrError::IncorrectNumberOfTypeParameters {
                            identifier: name,
                            expected: k.binders.len(),
                            actual: 0,
                        })
                    } else {
                        Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                            name: chalk_ir::TypeName::Struct(id),
                            substitution: chalk_ir::Substitution::empty(interner),
                        })
                        .intern(interner))
                    }
                }
                TypeLookup::Parameter(d) => Ok(chalk_ir::TyData::BoundVar(d).intern(interner)),
            },

            Ty::Dyn { ref bounds } => Ok(chalk_ir::TyData::Dyn(chalk_ir::DynTy {
                bounds: env.in_binders(
                    // FIXME: Figure out a proper name for this type parameter
                    Some(chalk_ir::ParameterKind::Ty(intern(FIXME_SELF))),
                    |env| {
                        Ok(bounds
                            .lower(env)?
                            .iter()
                            .flat_map(|qil| {
                                qil.into_where_clauses(
                                    interner,
                                    chalk_ir::TyData::BoundVar(0).intern(interner),
                                )
                            })
                            .collect())
                    },
                )?,
            })
            .intern(interner)),

            Ty::Apply { name, ref args } => {
                let id = match env.lookup_type(name)? {
                    TypeLookup::Struct(id) => id,
                    TypeLookup::Parameter(_) => Err(RustIrError::CannotApplyTypeParameter(name))?,
                };

                let k = env.struct_kind(id);
                if k.binders.len() != args.len() {
                    Err(RustIrError::IncorrectNumberOfTypeParameters {
                        identifier: name,
                        expected: k.binders.len(),
                        actual: args.len(),
                    })?;
                }

                let substitution = chalk_ir::Substitution::from_fallible(
                    interner,
                    args.iter().map(|t| Ok(t.lower(env)?)),
                )?;

                for (param, arg) in k.binders.binders.iter().zip(args.iter()) {
                    if param.kind() != arg.kind() {
                        Err(RustIrError::IncorrectParameterKind {
                            identifier: name,
                            expected: param.kind(),
                            actual: arg.kind(),
                        })?;
                    }
                }

                Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                    name: chalk_ir::TypeName::Struct(id),
                    substitution: substitution,
                })
                .intern(interner))
            }

            Ty::Alias { ref alias } => {
                Ok(chalk_ir::TyData::Alias(alias.lower(env)?).intern(interner))
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

                let function = chalk_ir::Fn {
                    num_binders: lifetime_names.len(),
                    parameters: vec![ty.lower(&quantified_env)?.cast(interner)],
                };
                Ok(chalk_ir::TyData::Function(function).intern(interner))
            }
        }
    }
}

trait LowerParameter {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Parameter<ChalkIr>>;
}

impl LowerParameter for Parameter {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Parameter<ChalkIr>> {
        let interner = env.interner();
        match *self {
            Parameter::Ty(ref t) => Ok(t.lower(env)?.cast(interner)),
            Parameter::Lifetime(ref l) => Ok(l.lower(env)?.cast(interner)),
        }
    }
}

trait LowerLifetime {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Lifetime<ChalkIr>>;
}

impl LowerLifetime for Lifetime {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Lifetime<ChalkIr>> {
        let interner = env.interner();
        match *self {
            Lifetime::Id { name } => match env.lookup_lifetime(name)? {
                LifetimeLookup::Parameter(d) => {
                    Ok(chalk_ir::LifetimeData::BoundVar(d).intern(interner))
                }
            },
        }
    }
}

trait LowerImpl {
    fn lower_impl(
        &self,
        empty_env: &Env,
        impl_id: ImplId<ChalkIr>,
        associated_ty_value_ids: &AssociatedTyValueIds,
    ) -> LowerResult<rust_ir::ImplDatum<ChalkIr>>;
}

impl LowerImpl for Impl {
    fn lower_impl(
        &self,
        empty_env: &Env,
        impl_id: ImplId<ChalkIr>,
        associated_ty_value_ids: &AssociatedTyValueIds,
    ) -> LowerResult<rust_ir::ImplDatum<ChalkIr>> {
        debug_heading!("LowerImpl::lower_impl(impl_id={:?})", impl_id);

        let polarity = self.polarity.lower();
        let binders = empty_env.in_binders(self.all_parameters(), |env| {
            let trait_ref = self.trait_ref.lower(env)?;
            debug!("trait_ref = {:?}", trait_ref);

            if !polarity.is_positive() && !self.assoc_ty_values.is_empty() {
                Err(RustIrError::NegativeImplAssociatedValues(
                    self.trait_ref.trait_name,
                ))?;
            }

            let where_clauses = self.lower_where_clauses(&env)?;
            debug!("where_clauses = {:?}", trait_ref);
            Ok(rust_ir::ImplDatumBound {
                trait_ref,
                where_clauses,
            })
        })?;

        // lookup the ids for each of the "associated type values"
        // within the impl, which should have already assigned and
        // stored in the map
        let associated_ty_value_ids = self
            .assoc_ty_values
            .iter()
            .map(|atv| associated_ty_value_ids[&(impl_id, atv.name.str)])
            .collect();

        debug!("associated_ty_value_ids = {:?}", associated_ty_value_ids);

        Ok(rust_ir::ImplDatum {
            polarity,
            binders: binders,
            impl_type: self.impl_type.lower(),
            associated_ty_value_ids,
        })
    }
}

trait LowerClause {
    fn lower_clause(&self, env: &Env) -> LowerResult<Vec<chalk_ir::ProgramClause<ChalkIr>>>;
}

impl LowerClause for Clause {
    fn lower_clause(&self, env: &Env) -> LowerResult<Vec<chalk_ir::ProgramClause<ChalkIr>>> {
        let interner = env.interner();
        let implications = env.in_binders(self.all_parameters(), |env| {
            let consequences: Vec<chalk_ir::DomainGoal<ChalkIr>> = self.consequence.lower(env)?;

            let conditions = chalk_ir::Goals::from_fallible(
                interner,
                // Subtle: in the SLG solver, we pop conditions from R to
                // L. To preserve the expected order (L to R), we must
                // therefore reverse.
                self.conditions.iter().map(|g| g.lower(env)).rev(),
            )?;

            let implications = consequences
                .into_iter()
                .map(|consequence| chalk_ir::ProgramClauseImplication {
                    consequence,
                    conditions: conditions.clone(),
                })
                .collect::<Vec<_>>();
            Ok(implications)
        })?;

        let clauses = implications
            .into_iter()
            .map(
                |implication: chalk_ir::Binders<chalk_ir::ProgramClauseImplication<ChalkIr>>| {
                    if implication.binders.is_empty() {
                        chalk_ir::ProgramClause::Implies(implication.value)
                    } else {
                        chalk_ir::ProgramClause::ForAll(implication)
                    }
                },
            )
            .collect();
        Ok(clauses)
    }
}

trait LowerTrait {
    fn lower_trait(
        &self,
        trait_id: chalk_ir::TraitId<ChalkIr>,
        env: &Env,
    ) -> LowerResult<rust_ir::TraitDatum<ChalkIr>>;
}

impl LowerTrait for TraitDefn {
    fn lower_trait(
        &self,
        trait_id: chalk_ir::TraitId<ChalkIr>,
        env: &Env,
    ) -> LowerResult<rust_ir::TraitDatum<ChalkIr>> {
        let all_parameters = self.all_parameters();
        let all_parameters_len = all_parameters.len();
        let binders = env.in_binders(all_parameters, |env| {
            if self.flags.auto {
                if all_parameters_len > 1 {
                    Err(RustIrError::AutoTraitParameters(self.name))?;
                }
                if !self.where_clauses.is_empty() {
                    Err(RustIrError::AutoTraitWhereClauses(self.name))?;
                }
            }

            Ok(rust_ir::TraitDatumBound {
                where_clauses: self.lower_where_clauses(env)?,
            })
        })?;

        let associated_ty_ids: Vec<_> = self
            .assoc_ty_defns
            .iter()
            .map(|defn| env.associated_ty_lookups[&(trait_id, defn.name.str)].id)
            .collect();

        Ok(rust_ir::TraitDatum {
            id: trait_id,
            binders: binders,
            flags: self.flags.lower(),
            associated_ty_ids,
            well_known: None,
        })
    }
}

pub trait LowerGoal<A> {
    fn lower(&self, arg: &A) -> LowerResult<chalk_ir::Goal<ChalkIr>>;
}

impl LowerGoal<LoweredProgram> for Goal {
    fn lower(&self, program: &LoweredProgram) -> LowerResult<chalk_ir::Goal<ChalkIr>> {
        let associated_ty_lookups: BTreeMap<_, _> = program
            .associated_ty_data
            .iter()
            .map(|(&associated_ty_id, datum)| {
                let trait_datum = &program.trait_data[&datum.trait_id];
                let num_trait_params = trait_datum.binders.len();
                let num_addl_params = datum.binders.len() - num_trait_params;
                let addl_parameter_kinds = datum.binders.binders[..num_addl_params].to_owned();
                let lookup = AssociatedTyLookup {
                    id: associated_ty_id,
                    addl_parameter_kinds,
                };
                ((datum.trait_id, datum.name), lookup)
            })
            .collect();

        let env = Env {
            struct_ids: &program.struct_ids,
            trait_ids: &program.trait_ids,
            struct_kinds: &program.struct_kinds,
            trait_kinds: &program.trait_kinds,
            associated_ty_lookups: &associated_ty_lookups,
            parameter_map: BTreeMap::new(),
        };

        self.lower(&env)
    }
}

impl<'k> LowerGoal<Env<'k>> for Goal {
    fn lower(&self, env: &Env<'k>) -> LowerResult<chalk_ir::Goal<ChalkIr>> {
        let interner = env.interner();
        match self {
            Goal::ForAll(ids, g) => g.lower_quantified(env, chalk_ir::QuantifierKind::ForAll, ids),
            Goal::Exists(ids, g) => g.lower_quantified(env, chalk_ir::QuantifierKind::Exists, ids),
            Goal::Implies(hyp, g) => {
                // We "elaborate" implied bounds by lowering goals like `T: Trait` and
                // `T: Trait<Assoc = U>` to `FromEnv(T: Trait)` and `FromEnv(T: Trait<Assoc = U>)`
                // in the assumptions of an `if` goal, e.g. `if (T: Trait) { ... }` lowers to
                // `if (FromEnv(T: Trait)) { ... /* this part is untouched */ ... }`.
                let where_clauses: LowerResult<Vec<_>> = hyp
                    .into_iter()
                    .flat_map(|h| h.lower_clause(env).apply_result())
                    .map(|result| result.map(|h| h.into_from_env_clause(interner)))
                    .collect();
                Ok(chalk_ir::GoalData::Implies(where_clauses?, g.lower(env)?).intern(interner))
            }
            Goal::And(g1, g2s) => {
                let goals = chalk_ir::Goals::from_fallible(
                    interner,
                    Some(g1).into_iter().chain(g2s).map(|g| g.lower(env)),
                )?;
                Ok(chalk_ir::GoalData::All(goals).intern(interner))
            }
            Goal::Not(g) => Ok(chalk_ir::GoalData::Not(g.lower(env)?).intern(interner)),
            Goal::Compatible(g) => Ok(g.lower(env)?.compatible(interner)),
            Goal::Leaf(leaf) => {
                // A where clause can lower to multiple leaf goals; wrap these in Goal::And.
                Ok(leaf.lower(env)?)
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
    ) -> LowerResult<chalk_ir::Goal<ChalkIr>>;
}

impl LowerQuantifiedGoal for Goal {
    fn lower_quantified(
        &self,
        env: &Env,
        quantifier_kind: chalk_ir::QuantifierKind,
        parameter_kinds: &[ParameterKind],
    ) -> LowerResult<chalk_ir::Goal<ChalkIr>> {
        let interner = env.interner();
        if parameter_kinds.is_empty() {
            return self.lower(env);
        }

        let parameter_kinds = parameter_kinds.iter().map(|pk| pk.lower());
        let subgoal = env.in_binders(parameter_kinds, |env| self.lower(env))?;
        Ok(chalk_ir::GoalData::Quantified(quantifier_kind, subgoal).intern(interner))
    }
}

/// Lowers LowerResult<Vec<T>> -> Vec<LowerResult<T>>.
trait ApplyResult {
    type Output;
    fn apply_result(self) -> Self::Output;
}

impl<T> ApplyResult for LowerResult<Vec<T>> {
    type Output = Vec<LowerResult<T>>;
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

#[allow(unreachable_code, unused_variables)]
impl Kinded for chalk_ir::Parameter<ChalkIr> {
    fn kind(&self) -> Kind {
        let interner = &ChalkIr;
        self.data(interner).kind()
    }
}
