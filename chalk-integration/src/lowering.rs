use crate::interner::{ChalkFnAbi, ChalkIr};
use chalk_ir::cast::{Cast, Caster};
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::{
    self, AdtId, AssocTypeId, BoundVar, ClausePriority, ClosureId, DebruijnIndex, FnDefId,
    ForeignDefId, ImplId, OpaqueTyId, QuantifiedWhereClauses, Substitution, TraitId, TyKind,
    VariableKinds,
};
use chalk_parse::ast::*;
use chalk_solve::rust_ir::{
    self, Anonymize, AssociatedTyValueId, IntoWhereClauses, OpaqueTyDatum, OpaqueTyDatumBound,
};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use string_cache::DefaultAtom as Atom;
use tracing::{debug, instrument};

use crate::error::RustIrError;
use crate::program::Program as LoweredProgram;
use crate::{Identifier as Ident, RawId, TypeKind, TypeSort};

type AdtIds = BTreeMap<Ident, chalk_ir::AdtId<ChalkIr>>;
type FnDefIds = BTreeMap<Ident, chalk_ir::FnDefId<ChalkIr>>;
type ClosureIds = BTreeMap<Ident, chalk_ir::ClosureId<ChalkIr>>;
type TraitIds = BTreeMap<Ident, chalk_ir::TraitId<ChalkIr>>;
type OpaqueTyIds = BTreeMap<Ident, chalk_ir::OpaqueTyId<ChalkIr>>;
type AdtKinds = BTreeMap<chalk_ir::AdtId<ChalkIr>, TypeKind>;
type FnDefKinds = BTreeMap<chalk_ir::FnDefId<ChalkIr>, TypeKind>;
type FnDefAbis = BTreeMap<FnDefId<ChalkIr>, <ChalkIr as Interner>::FnAbi>;
type ClosureKinds = BTreeMap<chalk_ir::ClosureId<ChalkIr>, TypeKind>;
type TraitKinds = BTreeMap<chalk_ir::TraitId<ChalkIr>, TypeKind>;
type AutoTraits = BTreeMap<chalk_ir::TraitId<ChalkIr>, bool>;
type OpaqueTyKinds = BTreeMap<chalk_ir::OpaqueTyId<ChalkIr>, TypeKind>;
type AssociatedTyLookups = BTreeMap<(chalk_ir::TraitId<ChalkIr>, Ident), AssociatedTyLookup>;
type AssociatedTyValueIds =
    BTreeMap<(chalk_ir::ImplId<ChalkIr>, Ident), AssociatedTyValueId<ChalkIr>>;
type ForeignIds = BTreeMap<Ident, chalk_ir::ForeignDefId<ChalkIr>>;

type ParameterMap = BTreeMap<Ident, chalk_ir::WithKind<ChalkIr, BoundVar>>;

pub type LowerResult<T> = Result<T, RustIrError>;

#[derive(Clone, Debug)]
struct Env<'k> {
    adt_ids: &'k AdtIds,
    adt_kinds: &'k AdtKinds,
    fn_def_ids: &'k FnDefIds,
    fn_def_kinds: &'k FnDefKinds,
    fn_def_abis: &'k FnDefAbis,
    closure_ids: &'k ClosureIds,
    closure_kinds: &'k ClosureKinds,
    trait_ids: &'k TraitIds,
    trait_kinds: &'k TraitKinds,
    opaque_ty_ids: &'k OpaqueTyIds,
    opaque_ty_kinds: &'k OpaqueTyKinds,
    associated_ty_lookups: &'k AssociatedTyLookups,
    auto_traits: &'k AutoTraits,
    foreign_ty_ids: &'k ForeignIds,
    /// GenericArg identifiers are used as keys, therefore
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
///          // addl_variable_kinds
/// }
/// ```
#[derive(Debug, PartialEq, Eq)]
struct AssociatedTyLookup {
    id: chalk_ir::AssocTypeId<ChalkIr>,
    addl_variable_kinds: Vec<chalk_ir::VariableKind<ChalkIr>>,
}

enum ApplyTypeLookup<'k> {
    Param(&'k chalk_ir::WithKind<ChalkIr, BoundVar>),
    Adt(AdtId<ChalkIr>),
    FnDef(FnDefId<ChalkIr>),
    Closure(ClosureId<ChalkIr>),
    Opaque(OpaqueTyId<ChalkIr>),
}

const SELF: &str = "Self";
const FIXME_SELF: &str = "__FIXME_SELF__";

impl Env<'_> {
    fn lookup_generic_arg(&self, name: &Identifier) -> LowerResult<chalk_ir::GenericArg<ChalkIr>> {
        let interner = self.interner();

        let apply = |k: &TypeKind, type_name: chalk_ir::TypeName<ChalkIr>| {
            if k.binders.len(interner) > 0 {
                Err(RustIrError::IncorrectNumberOfTypeParameters {
                    identifier: name.clone(),
                    expected: k.binders.len(interner),
                    actual: 0,
                })
            } else {
                Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                    name: type_name,
                    substitution: chalk_ir::Substitution::empty(interner),
                })
                .intern(interner)
                .cast(interner))
            }
        };

        match self.lookup_apply_type(name) {
            Ok(ApplyTypeLookup::Param(p)) => {
                let b = p.skip_kind();
                match &p.kind {
                    chalk_ir::VariableKind::Ty(_) => Ok(chalk_ir::TyData::BoundVar(*b)
                        .intern(interner)
                        .cast(interner)),
                    chalk_ir::VariableKind::Lifetime => Ok(chalk_ir::LifetimeData::BoundVar(*b)
                        .intern(interner)
                        .cast(interner)),
                    chalk_ir::VariableKind::Const(ty) => {
                        Ok(b.to_const(interner, ty.clone()).cast(interner))
                    }
                }
            }
            Ok(ApplyTypeLookup::Adt(id)) => apply(self.adt_kind(id), chalk_ir::TypeName::Adt(id)),
            Ok(ApplyTypeLookup::FnDef(id)) => {
                apply(self.fn_def_kind(id), chalk_ir::TypeName::FnDef(id))
            }
            Ok(ApplyTypeLookup::Closure(id)) => {
                apply(self.closure_kind(id), chalk_ir::TypeName::Closure(id))
            }
            Ok(ApplyTypeLookup::Opaque(id)) => Ok(chalk_ir::TyData::Alias(
                chalk_ir::AliasTy::Opaque(chalk_ir::OpaqueTy {
                    opaque_ty_id: id,
                    substitution: chalk_ir::Substitution::empty(interner),
                }),
            )
            .intern(interner)
            .cast(interner)),
            Err(_) => {
                if let Some(id) = self.foreign_ty_ids.get(&name.str) {
                    Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                        name: chalk_ir::TypeName::Foreign(*id),
                        substitution: chalk_ir::Substitution::empty(interner),
                    })
                    .intern(interner)
                    .cast(interner))
                } else if let Some(_) = self.trait_ids.get(&name.str) {
                    Err(RustIrError::NotStruct(name.clone()))
                } else {
                    Err(RustIrError::InvalidParameterName(name.clone()))
                }
            }
        }
    }

    fn lookup_apply_type(&self, name: &Identifier) -> LowerResult<ApplyTypeLookup> {
        if let Some(id) = self.parameter_map.get(&name.str) {
            return Ok(ApplyTypeLookup::Param(id));
        }

        if let Some(id) = self.adt_ids.get(&name.str) {
            return Ok(ApplyTypeLookup::Adt(*id));
        }

        if let Some(id) = self.fn_def_ids.get(&name.str) {
            return Ok(ApplyTypeLookup::FnDef(*id));
        }

        if let Some(id) = self.closure_ids.get(&name.str) {
            return Ok(ApplyTypeLookup::Closure(*id));
        }

        if let Some(id) = self.opaque_ty_ids.get(&name.str) {
            return Ok(ApplyTypeLookup::Opaque(*id));
        }

        Err(RustIrError::NotStruct(name.clone()))
    }

    fn auto_trait(&self, id: chalk_ir::TraitId<ChalkIr>) -> bool {
        self.auto_traits[&id]
    }

    fn lookup_trait(&self, name: &Identifier) -> LowerResult<TraitId<ChalkIr>> {
        if let Some(_) = self.parameter_map.get(&name.str) {
            return Err(RustIrError::NotTrait(name.clone()));
        }

        if let Some(_) = self.adt_ids.get(&name.str) {
            return Err(RustIrError::NotTrait(name.clone()));
        }

        if let Some(id) = self.trait_ids.get(&name.str) {
            return Ok(*id);
        }

        Err(RustIrError::InvalidTraitName(name.clone()))
    }

    fn trait_kind(&self, id: chalk_ir::TraitId<ChalkIr>) -> &TypeKind {
        &self.trait_kinds[&id]
    }

    fn adt_kind(&self, id: chalk_ir::AdtId<ChalkIr>) -> &TypeKind {
        &self.adt_kinds[&id]
    }

    fn fn_def_kind(&self, id: chalk_ir::FnDefId<ChalkIr>) -> &TypeKind {
        &self.fn_def_kinds[&id]
    }

    fn closure_kind(&self, id: chalk_ir::ClosureId<ChalkIr>) -> &TypeKind {
        &self.closure_kinds[&id]
    }

    fn opaque_kind(&self, id: chalk_ir::OpaqueTyId<ChalkIr>) -> &TypeKind {
        &self.opaque_ty_kinds[&id]
    }

    /// Introduces new parameters, shifting the indices of existing
    /// parameters to accommodate them. The indices of the new binders
    /// will be assigned in order as they are iterated.
    fn introduce<I>(&self, binders: I) -> LowerResult<Self>
    where
        I: IntoIterator<Item = chalk_ir::WithKind<ChalkIr, Ident>>,
        I::IntoIter: ExactSizeIterator,
    {
        // As binders to introduce we recieve `ParameterKind<Ident>`,
        // which we need to transform into `(Ident, ParameterKind<BoundVar>)`,
        // because that is the key-value pair for ParameterMap.
        // `swap_inner` lets us do precisely that, replacing `Ident` inside
        // `ParameterKind<Ident>` with a `BoundVar` and returning both.
        let binders = binders.into_iter().enumerate().map(|(i, k)| {
            let (kind, name) = k.into();
            (
                name,
                chalk_ir::WithKind::new(kind, BoundVar::new(DebruijnIndex::INNERMOST, i)),
            )
        });
        let len = binders.len();

        // For things already in the parameter map, we take each existing key-value pair
        // `(Ident, ParameterKind<BoundVar>)` and shift in the inner `BoundVar`.
        let parameter_map: ParameterMap = self
            .parameter_map
            .iter()
            .map(|(k, v)| (k.clone(), v.map_ref(|b| b.shifted_in())))
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
        I: IntoIterator<Item = chalk_ir::WithKind<ChalkIr, Ident>>,
        I::IntoIter: ExactSizeIterator,
        T: HasInterner<Interner = ChalkIr>,
        OP: FnOnce(&Self) -> LowerResult<T>,
    {
        let binders: Vec<_> = binders.into_iter().collect();
        let env = self.introduce(binders.iter().cloned())?;
        Ok(chalk_ir::Binders::new(
            VariableKinds::from_iter(self.interner(), binders.iter().map(|v| v.kind.clone())),
            op(&env)?,
        ))
    }
}

pub fn lower_program(program: &Program) -> LowerResult<LoweredProgram> {
    let mut index = 0;
    let mut next_item_id = || -> RawId {
        let i = index;
        index += 1;
        RawId { index: i }
    };

    // Make a vector mapping each thing in `items` to an id,
    // based just on its position:
    let raw_ids: Vec<_> = program.items.iter().map(|_| next_item_id()).collect();

    // Create ids for associated type declarations and values
    let mut associated_ty_lookups = BTreeMap::new();
    let mut associated_ty_value_ids = BTreeMap::new();
    for (item, &raw_id) in program.items.iter().zip(&raw_ids) {
        match item {
            Item::TraitDefn(d) => {
                if d.flags.auto && !d.assoc_ty_defns.is_empty() {
                    Err(RustIrError::AutoTraitAssociatedTypes(d.name.clone()))?;
                }
                for defn in &d.assoc_ty_defns {
                    let addl_variable_kinds = defn.all_parameters();
                    let lookup = AssociatedTyLookup {
                        id: AssocTypeId(next_item_id()),
                        addl_variable_kinds: addl_variable_kinds.anonymize(),
                    };
                    associated_ty_lookups.insert((TraitId(raw_id), defn.name.str.clone()), lookup);
                }
            }

            Item::Impl(d) => {
                for atv in &d.assoc_ty_values {
                    let atv_id = AssociatedTyValueId(next_item_id());
                    associated_ty_value_ids.insert((ImplId(raw_id), atv.name.str.clone()), atv_id);
                }
            }

            _ => {}
        }
    }

    let mut adt_ids = BTreeMap::new();
    let mut fn_def_ids = BTreeMap::new();
    let mut closure_ids = BTreeMap::new();
    let mut trait_ids = BTreeMap::new();
    let mut auto_traits = BTreeMap::new();
    let mut opaque_ty_ids = BTreeMap::new();
    let mut adt_kinds = BTreeMap::new();
    let mut fn_def_kinds = BTreeMap::new();
    let mut fn_def_abis = BTreeMap::new();
    let mut closure_kinds = BTreeMap::new();
    let mut trait_kinds = BTreeMap::new();
    let mut opaque_ty_kinds = BTreeMap::new();
    let mut object_safe_traits = HashSet::new();

    for (item, &raw_id) in program.items.iter().zip(&raw_ids) {
        match item {
            Item::AdtDefn(defn) => {
                let type_kind = defn.lower_type_kind()?;
                let id = AdtId(raw_id);
                adt_ids.insert(type_kind.name.clone(), id);
                adt_kinds.insert(id, type_kind);
            }
            Item::FnDefn(defn) => {
                let type_kind = defn.lower_type_kind()?;
                let id = FnDefId(raw_id);
                fn_def_ids.insert(type_kind.name.clone(), id);
                fn_def_kinds.insert(id, type_kind);
                fn_def_abis.insert(id, lower_fn_abi(&defn.sig.abi)?);
            }
            Item::ClosureDefn(defn) => {
                let type_kind = defn.lower_type_kind()?;
                let id = ClosureId(raw_id);
                closure_ids.insert(defn.name.str.clone(), id);
                closure_kinds.insert(id, type_kind);
            }
            Item::TraitDefn(defn) => {
                let type_kind = defn.lower_type_kind()?;
                let id = TraitId(raw_id);
                trait_ids.insert(type_kind.name.clone(), id);
                trait_kinds.insert(id, type_kind);
                auto_traits.insert(id, defn.flags.auto);

                if defn.flags.object_safe {
                    object_safe_traits.insert(id);
                }
            }
            Item::OpaqueTyDefn(defn) => {
                let type_kind = defn.lower_type_kind()?;
                let id = OpaqueTyId(raw_id);
                opaque_ty_ids.insert(defn.name.str.clone(), id);
                opaque_ty_kinds.insert(id, type_kind);
            }
            Item::Impl(_) => continue,
            Item::Clause(_) => continue,
            Item::Foreign(_) => continue,
        };
    }

    let mut adt_data = BTreeMap::new();
    let mut adt_reprs = BTreeMap::new();
    let mut fn_def_data = BTreeMap::new();
    let mut closure_inputs_and_output = BTreeMap::new();
    let mut closure_closure_kind = BTreeMap::new();
    let mut closure_upvars = BTreeMap::new();
    let mut trait_data = BTreeMap::new();
    let mut well_known_traits = BTreeMap::new();
    let mut impl_data = BTreeMap::new();
    let mut associated_ty_data = BTreeMap::new();
    let mut associated_ty_values = BTreeMap::new();
    let mut opaque_ty_data = BTreeMap::new();
    let mut hidden_opaque_types = BTreeMap::new();
    let mut custom_clauses = Vec::new();
    let mut foreign_ty_ids = BTreeMap::new();

    for (item, &raw_id) in program.items.iter().zip(&raw_ids) {
        let empty_env = Env {
            adt_ids: &adt_ids,
            adt_kinds: &adt_kinds,
            fn_def_ids: &fn_def_ids,
            fn_def_kinds: &fn_def_kinds,
            fn_def_abis: &fn_def_abis,
            closure_ids: &closure_ids,
            closure_kinds: &closure_kinds,
            trait_ids: &trait_ids,
            trait_kinds: &trait_kinds,
            opaque_ty_ids: &opaque_ty_ids,
            opaque_ty_kinds: &opaque_ty_kinds,
            associated_ty_lookups: &associated_ty_lookups,
            parameter_map: BTreeMap::new(),
            auto_traits: &auto_traits,
            foreign_ty_ids: &foreign_ty_ids,
        };

        match *item {
            Item::AdtDefn(ref d) => {
                let adt_id = AdtId(raw_id);
                adt_data.insert(adt_id, Arc::new(lower_adt(d, adt_id, &empty_env)?));
                adt_reprs.insert(adt_id, lower_adt_repr(&d.repr));
            }
            Item::FnDefn(ref defn) => {
                let fn_def_id = FnDefId(raw_id);
                fn_def_data.insert(
                    fn_def_id,
                    Arc::new(lower_fn_def(defn, fn_def_id, &empty_env)?),
                );
            }
            Item::ClosureDefn(ref defn) => {
                let closure_def_id = ClosureId(raw_id);
                let (kind, inputs_and_output) = defn.lower(&empty_env)?;
                closure_closure_kind.insert(closure_def_id, kind);
                closure_inputs_and_output.insert(closure_def_id, inputs_and_output);
                let upvars = empty_env.in_binders(defn.all_parameters(), |env| {
                    let upvar_tys: LowerResult<Vec<chalk_ir::Ty<ChalkIr>>> =
                        defn.upvars.iter().map(|ty| ty.lower(&env)).collect();
                    let substitution = chalk_ir::Substitution::from_iter(
                        &ChalkIr,
                        upvar_tys?.into_iter().map(|ty| ty.cast(&ChalkIr)),
                    );
                    Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                        name: chalk_ir::TypeName::Tuple(defn.upvars.len()),
                        substitution,
                    })
                    .intern(&ChalkIr))
                })?;
                closure_upvars.insert(closure_def_id, upvars);
            }
            Item::TraitDefn(ref trait_defn) => {
                let trait_id = TraitId(raw_id);
                let trait_datum = lower_trait(trait_defn, trait_id, &empty_env)?;

                if let Some(well_known) = trait_datum.well_known {
                    well_known_traits.insert(well_known, trait_id);
                }

                trait_data.insert(trait_id, Arc::new(trait_datum));

                for assoc_ty_defn in &trait_defn.assoc_ty_defns {
                    let lookup =
                        &associated_ty_lookups[&(trait_id, assoc_ty_defn.name.str.clone())];

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
                    let mut variable_kinds = assoc_ty_defn.all_parameters();
                    variable_kinds.extend(trait_defn.all_parameters());

                    let binders = empty_env.in_binders(variable_kinds, |env| {
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
                            name: assoc_ty_defn.name.str.clone(),
                            binders,
                        }),
                    );
                }
            }
            Item::Impl(ref impl_defn) => {
                let impl_id = ImplId(raw_id);
                let impl_datum = Arc::new(lower_impl(
                    impl_defn,
                    &empty_env,
                    impl_id,
                    &associated_ty_value_ids,
                )?);
                impl_data.insert(impl_id, impl_datum.clone());
                let trait_id = impl_datum.trait_id();

                for atv in &impl_defn.assoc_ty_values {
                    let atv_id = associated_ty_value_ids[&(impl_id, atv.name.str.clone())];
                    let lookup = &associated_ty_lookups[&(trait_id, atv.name.str.clone())];

                    // The parameters in scope for the associated
                    // type definitions are *both* those from the
                    // impl *and* those from the associated type
                    // itself. As in the "trait" case above, we begin
                    // with the parameters from the impl.
                    let mut variable_kinds = atv.all_parameters();
                    variable_kinds.extend(impl_defn.all_parameters());

                    let value = empty_env.in_binders(variable_kinds, |env| {
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
                custom_clauses.extend(clause.lower(&empty_env)?);
            }
            Item::OpaqueTyDefn(ref opaque_ty) => {
                if let Some(&opaque_ty_id) = opaque_ty_ids.get(&opaque_ty.name.str) {
                    let variable_kinds = opaque_ty
                        .variable_kinds
                        .iter()
                        .map(lower_variable_kind)
                        .collect::<Vec<_>>();

                    // Introduce the parameters declared on the opaque type definition.
                    // So if we have `type Foo<P1..Pn> = impl Trait<T1..Tn>`, this would introduce `P1..Pn`
                    let binders = empty_env.in_binders(variable_kinds, |env| {
                        let hidden_ty = opaque_ty.ty.lower(&env)?;
                        hidden_opaque_types.insert(opaque_ty_id, Arc::new(hidden_ty));

                        // Introduce a variable to represent the hidden "self type". This will be used in the bounds.
                        // So the `impl Trait<T1..Tn>` will be lowered to `exists<Self> { Self: Trait<T1..Tn> }`.
                        let bounds: chalk_ir::Binders<Vec<chalk_ir::Binders<_>>> = env.in_binders(
                            Some(chalk_ir::WithKind::new(
                                chalk_ir::VariableKind::Ty(TyKind::General),
                                Atom::from(FIXME_SELF),
                            )),
                            |env| {
                                let interner = env.interner();
                                Ok(opaque_ty
                                    .bounds
                                    .lower(&env)?
                                    .iter()
                                    .flat_map(|qil| {
                                        // Instantiate the bounds with the innermost bound variable, which represents Self, as the self type.
                                        qil.into_where_clauses(
                                            interner,
                                            chalk_ir::TyData::BoundVar(BoundVar::new(
                                                DebruijnIndex::INNERMOST,
                                                0,
                                            ))
                                            .intern(interner),
                                        )
                                    })
                                    .collect())
                            },
                        )?;
                        let where_clauses: chalk_ir::Binders<Vec<chalk_ir::Binders<_>>> = env
                            .in_binders(
                                Some(chalk_ir::WithKind::new(
                                    chalk_ir::VariableKind::Ty(TyKind::General),
                                    Atom::from(FIXME_SELF),
                                )),
                                |env| opaque_ty.where_clauses.lower(env),
                            )?;

                        Ok(OpaqueTyDatumBound {
                            bounds,
                            where_clauses,
                        })
                    })?;

                    opaque_ty_data.insert(
                        opaque_ty_id,
                        Arc::new(OpaqueTyDatum {
                            opaque_ty_id,
                            bound: binders,
                        }),
                    );
                }
            }
            Item::Foreign(ForeignDefn(ref ident)) => {
                foreign_ty_ids.insert(ident.str.clone(), ForeignDefId(raw_id));
            }
        }
    }

    let program = LoweredProgram {
        adt_ids,
        fn_def_ids,
        closure_ids,
        closure_upvars,
        closure_kinds,
        trait_ids,
        adt_kinds,
        fn_def_kinds,
        trait_kinds,
        adt_data,
        adt_reprs,
        fn_def_data,
        closure_inputs_and_output,
        closure_closure_kind,
        trait_data,
        well_known_traits,
        impl_data,
        associated_ty_values,
        associated_ty_data,
        opaque_ty_ids,
        opaque_ty_kinds,
        opaque_ty_data,
        hidden_opaque_types,
        custom_clauses,
        object_safe_traits,
        foreign_ty_ids,
    };

    Ok(program)
}

trait LowerParameterMap {
    fn synthetic_parameters(&self) -> Option<chalk_ir::WithKind<ChalkIr, Ident>>;
    fn declared_parameters(&self) -> &[VariableKind];
    fn all_parameters(&self) -> Vec<chalk_ir::WithKind<ChalkIr, Ident>> {
        self.synthetic_parameters()
            .into_iter()
            .chain(self.declared_parameters().iter().map(lower_variable_kind))
            .collect()

        /* TODO: switch to this ordering, but adjust *all* the code to match

        self.declared_parameters()
            .iter()
            .map(|id| id.lower())
            .chain(self.synthetic_parameters()) // (*) see below
            .collect()
         */

        // (*) It is important that the declared parameters come
        // before the synthetic parameters in the ordering. This is
        // because of traits, when used as types, only have the first
        // N parameters in their kind (that is, they do not have Self).
        //
        // Note that if `Self` appears in the where-clauses etc, the
        // trait is not object-safe, and hence not supposed to be used
        // as an object. Actually the handling of object types is
        // probably just kind of messed up right now. That's ok.
    }
}

macro_rules! lower_param_map {
    ($type: ident, $synthetic: expr) => {
        impl LowerParameterMap for $type {
            fn synthetic_parameters(&self) -> Option<chalk_ir::WithKind<ChalkIr, Ident>> {
                $synthetic
            }
            fn declared_parameters(&self) -> &[VariableKind] {
                &self.variable_kinds
            }
        }
    };
}
lower_param_map!(AdtDefn, None);
lower_param_map!(FnDefn, None);
lower_param_map!(ClosureDefn, None);
lower_param_map!(Impl, None);
lower_param_map!(AssocTyDefn, None);
lower_param_map!(AssocTyValue, None);
lower_param_map!(Clause, None);
lower_param_map!(
    TraitDefn,
    Some(chalk_ir::WithKind::new(
        chalk_ir::VariableKind::Ty(TyKind::General),
        Atom::from(SELF),
    ))
);

trait LowerTypeKind {
    fn lower_type_kind(&self) -> LowerResult<TypeKind>;
}

macro_rules! lower_type_kind {
    ($type: ident, $sort: ident, $params: expr) => {
        impl LowerTypeKind for $type {
            fn lower_type_kind(&self) -> LowerResult<TypeKind> {
                Ok(TypeKind {
                    sort: TypeSort::$sort,
                    name: self.name.str.clone(),
                    binders: chalk_ir::Binders::new(
                        VariableKinds::from_iter(&ChalkIr, $params(self).anonymize()),
                        crate::Unit,
                    ),
                })
            }
        }
    };
}

lower_type_kind!(AdtDefn, Adt, |defn: &AdtDefn| defn.all_parameters());
lower_type_kind!(FnDefn, FnDef, |defn: &FnDefn| defn.all_parameters());
lower_type_kind!(ClosureDefn, Closure, |defn: &ClosureDefn| defn
    .all_parameters());
lower_type_kind!(TraitDefn, Trait, |defn: &TraitDefn| defn
    .variable_kinds
    .iter()
    .map(lower_variable_kind)
    .collect::<Vec<_>>());
lower_type_kind!(OpaqueTyDefn, Opaque, |defn: &OpaqueTyDefn| defn
    .variable_kinds
    .iter()
    .map(lower_variable_kind)
    .collect::<Vec<_>>());

fn get_type_of_u32() -> chalk_ir::Ty<ChalkIr> {
    chalk_ir::ApplicationTy {
        name: chalk_ir::TypeName::Scalar(chalk_ir::Scalar::Uint(chalk_ir::UintTy::U32)),
        substitution: Substitution::empty(&ChalkIr),
    }
    .cast(&ChalkIr)
    .intern(&ChalkIr)
}

fn lower_variable_kind(variable_kind: &VariableKind) -> chalk_ir::WithKind<ChalkIr, Ident> {
    let (kind, n) = match variable_kind {
        VariableKind::Ty(n) => (chalk_ir::VariableKind::Ty(chalk_ir::TyKind::General), n),
        VariableKind::IntegerTy(n) => (chalk_ir::VariableKind::Ty(chalk_ir::TyKind::Integer), n),
        VariableKind::FloatTy(n) => (chalk_ir::VariableKind::Ty(chalk_ir::TyKind::Float), n),
        VariableKind::Lifetime(n) => (chalk_ir::VariableKind::Lifetime, n),
        VariableKind::Const(ref n) => (chalk_ir::VariableKind::Const(get_type_of_u32()), n),
    };

    chalk_ir::WithKind::new(kind, n.str.clone())
}

trait Lower {
    type Lowered;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered>;
}

impl Lower for [QuantifiedWhereClause] {
    type Lowered = Vec<chalk_ir::QuantifiedWhereClause<ChalkIr>>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        self.iter()
            .flat_map(|wc| match wc.lower(env) {
                Ok(v) => v.into_iter().map(Ok).collect(),
                Err(e) => vec![Err(e)],
            })
            .collect()
    }
}

impl Lower for WhereClause {
    type Lowered = Vec<chalk_ir::WhereClause<ChalkIr>>;

    /// Lower from an AST `where` clause to an internal IR.
    /// Some AST `where` clauses can lower to multiple ones, this is why we return a `Vec`.
    /// As for now, this is the only the case for `where T: Foo<Item = U>` which lowers to
    /// `Implemented(T: Foo)` and `ProjectionEq(<T as Foo>::Item = U)`.
    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let where_clauses = match self {
            WhereClause::Implemented { trait_ref } => {
                vec![chalk_ir::WhereClause::Implemented(trait_ref.lower(env)?)]
            }
            WhereClause::ProjectionEq { projection, ty } => vec![
                chalk_ir::WhereClause::AliasEq(chalk_ir::AliasEq {
                    alias: chalk_ir::AliasTy::Projection(projection.lower(env)?),
                    ty: ty.lower(env)?,
                }),
                chalk_ir::WhereClause::Implemented(projection.trait_ref.lower(env)?),
            ],
            WhereClause::LifetimeOutlives { a, b } => {
                vec![chalk_ir::WhereClause::LifetimeOutlives(
                    chalk_ir::LifetimeOutlives {
                        a: a.lower(env)?,
                        b: b.lower(env)?,
                    },
                )]
            }
            WhereClause::TypeOutlives { ty, lifetime } => {
                vec![chalk_ir::WhereClause::TypeOutlives(
                    chalk_ir::TypeOutlives {
                        ty: ty.lower(env)?,
                        lifetime: lifetime.lower(env)?,
                    },
                )]
            }
        };
        Ok(where_clauses)
    }
}

impl Lower for QuantifiedWhereClause {
    type Lowered = Vec<chalk_ir::QuantifiedWhereClause<ChalkIr>>;

    /// Lower from an AST `where` clause to an internal IR.
    /// Some AST `where` clauses can lower to multiple ones, this is why we return a `Vec`.
    /// As for now, this is the only the case for `where T: Foo<Item = U>` which lowers to
    /// `Implemented(T: Foo)` and `ProjectionEq(<T as Foo>::Item = U)`.
    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let variable_kinds = self.variable_kinds.iter().map(lower_variable_kind);
        let binders = env.in_binders(variable_kinds, |env| Ok(self.where_clause.lower(env)?))?;
        Ok(binders.into_iter().collect())
    }
}

impl Lower for DomainGoal {
    type Lowered = Vec<chalk_ir::DomainGoal<ChalkIr>>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let interner = env.interner();
        let goals = match self {
            DomainGoal::Holds { where_clause } => where_clause
                .lower(env)?
                .into_iter()
                .casted(interner)
                .collect(),
            DomainGoal::Normalize { projection, ty } => {
                vec![chalk_ir::DomainGoal::Normalize(chalk_ir::Normalize {
                    alias: chalk_ir::AliasTy::Projection(projection.lower(env)?),
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
            DomainGoal::Compatible => vec![chalk_ir::DomainGoal::Compatible],
            DomainGoal::DownstreamType { ty } => {
                vec![chalk_ir::DomainGoal::DownstreamType(ty.lower(env)?)]
            }
            DomainGoal::Reveal => vec![chalk_ir::DomainGoal::Reveal],
            DomainGoal::ObjectSafe { id } => {
                vec![chalk_ir::DomainGoal::ObjectSafe(env.lookup_trait(id)?)]
            }
        };
        Ok(goals)
    }
}

impl Lower for LeafGoal {
    type Lowered = chalk_ir::Goal<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let interner = env.interner();
        Ok(match self {
            LeafGoal::DomainGoal { goal } => {
                chalk_ir::Goal::all(interner, goal.lower(env)?.into_iter().casted(interner))
            }
            LeafGoal::UnifyGenericArgs { a, b } => chalk_ir::EqGoal {
                a: a.lower(env)?.cast(interner),
                b: b.lower(env)?.cast(interner),
            }
            .cast::<chalk_ir::Goal<ChalkIr>>(interner),
        })
    }
}

fn lower_adt(
    adt_defn: &AdtDefn,
    adt_id: chalk_ir::AdtId<ChalkIr>,
    env: &Env,
) -> LowerResult<rust_ir::AdtDatum<ChalkIr>> {
    if adt_defn.flags.fundamental && adt_defn.all_parameters().len() != 1 {
        Err(RustIrError::InvalidFundamentalTypesParameters(
            adt_defn.name.clone(),
        ))?;
    }

    let binders = env.in_binders(adt_defn.all_parameters(), |env| {
        Ok(rust_ir::AdtDatumBound {
            variants: adt_defn
                .variants
                .iter()
                .map(|v| {
                    let fields: LowerResult<_> = v.fields.iter().map(|f| f.ty.lower(env)).collect();
                    Ok(rust_ir::AdtVariantDatum { fields: fields? })
                })
                .collect::<LowerResult<_>>()?,
            where_clauses: adt_defn.where_clauses.lower(env)?,
        })
    })?;

    let flags = rust_ir::AdtFlags {
        upstream: adt_defn.flags.upstream,
        fundamental: adt_defn.flags.fundamental,
        phantom_data: adt_defn.flags.phantom_data,
    };

    Ok(rust_ir::AdtDatum {
        id: adt_id,
        binders,
        flags,
        kind: match adt_defn.flags.kind {
            AdtKind::Struct => rust_ir::AdtKind::Struct,
            AdtKind::Enum => rust_ir::AdtKind::Enum,
            AdtKind::Union => rust_ir::AdtKind::Union,
        },
    })
}

fn lower_adt_repr(adt_repr: &AdtRepr) -> rust_ir::AdtRepr {
    rust_ir::AdtRepr {
        repr_c: adt_repr.repr_c,
        repr_packed: adt_repr.repr_packed,
    }
}

fn lower_fn_def(
    fn_defn: &FnDefn,
    fn_def_id: chalk_ir::FnDefId<ChalkIr>,
    env: &Env,
) -> LowerResult<rust_ir::FnDefDatum<ChalkIr>> {
    let binders = env.in_binders(fn_defn.all_parameters(), |env| {
        let where_clauses = fn_defn.where_clauses.lower(env)?;

        let inputs_and_output = env.in_binders(vec![], |env| {
            let args: LowerResult<_> = fn_defn
                .argument_types
                .iter()
                .map(|t| t.lower(env))
                .collect();
            let return_type = fn_defn.return_type.lower(env)?;
            Ok(rust_ir::FnDefInputsAndOutputDatum {
                argument_types: args?,
                return_type,
            })
        })?;
        Ok(rust_ir::FnDefDatumBound {
            inputs_and_output,
            where_clauses,
        })
    })?;

    Ok(rust_ir::FnDefDatum {
        id: fn_def_id,
        sig: lower_fn_sig(&fn_defn.sig)?,
        binders,
    })
}

fn lower_fn_sig(fn_sig: &FnSig) -> LowerResult<chalk_ir::FnSig<ChalkIr>> {
    Ok(chalk_ir::FnSig {
        abi: lower_fn_abi(&fn_sig.abi)?,
        safety: lower_safety(&fn_sig.safety),
        variadic: fn_sig.variadic,
    })
}

fn lower_fn_abi(fn_abi: &FnAbi) -> LowerResult<ChalkFnAbi> {
    match fn_abi.0.as_ref() {
        "Rust" => Ok(ChalkFnAbi::Rust),
        "C" => Ok(ChalkFnAbi::C),
        _ => Err(RustIrError::InvalidExternAbi(fn_abi.0.clone())),
    }
}

impl Lower for ClosureDefn {
    type Lowered = (
        rust_ir::ClosureKind,
        chalk_ir::Binders<rust_ir::FnDefInputsAndOutputDatum<ChalkIr>>,
    );

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let inputs_and_output = env.in_binders(self.all_parameters(), |env| {
            let args: LowerResult<_> = self.argument_types.iter().map(|t| t.lower(env)).collect();
            let return_type = self.return_type.lower(env)?;
            Ok(rust_ir::FnDefInputsAndOutputDatum {
                argument_types: args?,
                return_type,
            })
        })?;

        Ok((lower_closure_kind(&self.kind), inputs_and_output))
    }
}

fn lower_closure_kind(closure_kind: &ClosureKind) -> rust_ir::ClosureKind {
    match closure_kind {
        ClosureKind::Fn => rust_ir::ClosureKind::Fn,
        ClosureKind::FnMut => rust_ir::ClosureKind::FnMut,
        ClosureKind::FnOnce => rust_ir::ClosureKind::FnOnce,
    }
}

impl Lower for TraitRef {
    type Lowered = chalk_ir::TraitRef<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let interner = env.interner();
        let without_self = TraitBound {
            trait_name: self.trait_name.clone(),
            args_no_self: self.args.iter().cloned().skip(1).collect(),
        }
        .lower(env)?;

        let self_parameter = self.args[0].lower(env)?;
        Ok(without_self.as_trait_ref(interner, self_parameter.assert_ty_ref(interner).clone()))
    }
}

impl Lower for TraitBound {
    type Lowered = rust_ir::TraitBound<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let interner = env.interner();
        let trait_id = env.lookup_trait(&self.trait_name)?;

        let k = env.trait_kind(trait_id);
        if k.sort != TypeSort::Trait {
            Err(RustIrError::NotTrait(self.trait_name.clone()))?;
        }

        let parameters = self
            .args_no_self
            .iter()
            .map(|a| Ok(a.lower(env)?))
            .collect::<LowerResult<Vec<_>>>()?;

        if parameters.len() != k.binders.len(interner) {
            Err(RustIrError::IncorrectNumberOfTypeParameters {
                identifier: self.trait_name.clone(),
                expected: k.binders.len(interner),
                actual: parameters.len(),
            })?;
        }

        for (binder, param) in k.binders.binders.iter(interner).zip(parameters.iter()) {
            if binder.kind() != param.kind() {
                Err(RustIrError::IncorrectTraitParameterKind {
                    identifier: self.trait_name.clone(),
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

impl Lower for AliasEqBound {
    type Lowered = rust_ir::AliasEqBound<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let trait_bound = self.trait_bound.lower(env)?;
        let lookup = match env
            .associated_ty_lookups
            .get(&(trait_bound.trait_id, self.name.str.clone()))
        {
            Some(lookup) => lookup,
            None => Err(RustIrError::MissingAssociatedType(self.name.clone()))?,
        };
        let args: Vec<_> = self
            .args
            .iter()
            .map(|a| a.lower(env))
            .collect::<LowerResult<_>>()?;

        if args.len() != lookup.addl_variable_kinds.len() {
            Err(RustIrError::IncorrectNumberOfAssociatedTypeParameters {
                identifier: self.name.clone(),
                expected: lookup.addl_variable_kinds.len(),
                actual: args.len(),
            })?;
        }

        for (param, arg) in lookup.addl_variable_kinds.iter().zip(args.iter()) {
            if param.kind() != arg.kind() {
                Err(RustIrError::IncorrectAssociatedTypeParameterKind {
                    identifier: self.name.clone(),
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

impl Lower for InlineBound {
    type Lowered = rust_ir::InlineBound<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let bound = match self {
            InlineBound::TraitBound(b) => rust_ir::InlineBound::TraitBound(b.lower(&env)?),
            InlineBound::AliasEqBound(b) => rust_ir::InlineBound::AliasEqBound(b.lower(&env)?),
        };
        Ok(bound)
    }
}

impl Lower for QuantifiedInlineBound {
    type Lowered = rust_ir::QuantifiedInlineBound<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let variable_kinds = self.variable_kinds.iter().map(lower_variable_kind);
        let binders = env.in_binders(variable_kinds, |env| Ok(self.bound.lower(env)?))?;
        Ok(binders)
    }
}

impl Lower for [QuantifiedInlineBound] {
    type Lowered = Vec<rust_ir::QuantifiedInlineBound<ChalkIr>>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        fn trait_identifier(bound: &InlineBound) -> &Identifier {
            match bound {
                InlineBound::TraitBound(tb) => &tb.trait_name,
                InlineBound::AliasEqBound(ab) => &ab.trait_bound.trait_name,
            }
        }

        let mut regular_traits = Vec::new();
        let mut auto_traits = Vec::new();

        for b in self {
            let id = env.lookup_trait(trait_identifier(&b.bound))?;
            if env.auto_trait(id) {
                auto_traits.push((b, id))
            } else {
                regular_traits.push((b, id))
            }
        }

        auto_traits.sort_by_key(|b| b.1);

        regular_traits
            .iter()
            .chain(auto_traits.iter())
            .map(|(b, _)| b.lower(env))
            .collect()
    }
}

fn lower_polarity(polarity: &Polarity) -> rust_ir::Polarity {
    match polarity {
        Polarity::Positive => rust_ir::Polarity::Positive,
        Polarity::Negative => rust_ir::Polarity::Negative,
    }
}

fn lower_impl_type(impl_type: &ImplType) -> rust_ir::ImplType {
    match impl_type {
        ImplType::Local => rust_ir::ImplType::Local,
        ImplType::External => rust_ir::ImplType::External,
    }
}

fn lower_trait_flags(trait_flags: &TraitFlags) -> rust_ir::TraitFlags {
    rust_ir::TraitFlags {
        auto: trait_flags.auto,
        marker: trait_flags.marker,
        upstream: trait_flags.upstream,
        fundamental: trait_flags.fundamental,
        non_enumerable: trait_flags.non_enumerable,
        coinductive: trait_flags.coinductive,
    }
}

impl Lower for ProjectionTy {
    type Lowered = chalk_ir::ProjectionTy<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let ProjectionTy {
            ref trait_ref,
            ref name,
            ref args,
        } = *self;
        let interner = env.interner();
        let chalk_ir::TraitRef {
            trait_id,
            substitution: trait_substitution,
        } = trait_ref.lower(env)?;
        let lookup = match env.associated_ty_lookups.get(&(trait_id, name.str.clone())) {
            Some(lookup) => lookup,
            None => Err(RustIrError::MissingAssociatedType(self.name.clone()))?,
        };
        let mut args: Vec<_> = args
            .iter()
            .map(|a| a.lower(env))
            .collect::<LowerResult<_>>()?;

        if args.len() != lookup.addl_variable_kinds.len() {
            Err(RustIrError::IncorrectNumberOfAssociatedTypeParameters {
                identifier: self.name.clone(),
                expected: lookup.addl_variable_kinds.len(),
                actual: args.len(),
            })?;
        }

        for (param, arg) in lookup.addl_variable_kinds.iter().zip(args.iter()) {
            if param.kind() != arg.kind() {
                Err(RustIrError::IncorrectAssociatedTypeParameterKind {
                    identifier: self.name.clone(),
                    expected: param.kind(),
                    actual: arg.kind(),
                })?;
            }
        }

        args.extend(trait_substitution.iter(interner).cloned());

        Ok(chalk_ir::ProjectionTy {
            associated_ty_id: lookup.id,
            substitution: chalk_ir::Substitution::from_iter(interner, args),
        })
    }
}

impl Lower for Ty {
    type Lowered = chalk_ir::Ty<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let interner = env.interner();
        match self {
            Ty::Id { name } => {
                let parameter = env.lookup_generic_arg(&name)?;
                parameter.ty(interner).map(|ty| ty.clone()).ok_or_else(|| {
                    RustIrError::IncorrectParameterKind {
                        identifier: name.clone(),
                        expected: Kind::Ty,
                        actual: parameter.kind(),
                    }
                })
            }
            Ty::Dyn {
                ref bounds,
                ref lifetime,
            } => Ok(chalk_ir::TyData::Dyn(chalk_ir::DynTy {
                bounds: env.in_binders(
                    // FIXME: Figure out a proper name for this type parameter
                    Some(chalk_ir::WithKind::new(
                        chalk_ir::VariableKind::Ty(TyKind::General),
                        Atom::from(FIXME_SELF),
                    )),
                    |env| {
                        Ok(QuantifiedWhereClauses::from_iter(
                            interner,
                            bounds.lower(env)?.iter().flat_map(|qil| {
                                qil.into_where_clauses(
                                    interner,
                                    chalk_ir::TyData::BoundVar(BoundVar::new(
                                        DebruijnIndex::INNERMOST,
                                        0,
                                    ))
                                    .intern(interner),
                                )
                            }),
                        ))
                    },
                )?,
                lifetime: lifetime.lower(env)?,
            })
            .intern(interner)),

            Ty::Apply { name, ref args } => {
                let (apply_name, k) = match env.lookup_apply_type(&name)? {
                    ApplyTypeLookup::Param(_) => {
                        return Err(RustIrError::CannotApplyTypeParameter(name.clone()))
                    }

                    ApplyTypeLookup::Adt(id) => (chalk_ir::TypeName::Adt(id), env.adt_kind(id)),
                    ApplyTypeLookup::FnDef(id) => {
                        (chalk_ir::TypeName::FnDef(id), env.fn_def_kind(id))
                    }
                    ApplyTypeLookup::Closure(id) => {
                        (chalk_ir::TypeName::Closure(id), env.closure_kind(id))
                    }
                    ApplyTypeLookup::Opaque(id) => {
                        (chalk_ir::TypeName::OpaqueType(id), env.opaque_kind(id))
                    }
                };

                if k.binders.len(interner) != args.len() {
                    Err(RustIrError::IncorrectNumberOfTypeParameters {
                        identifier: name.clone(),
                        expected: k.binders.len(interner),
                        actual: args.len(),
                    })?;
                }

                let substitution = chalk_ir::Substitution::from_fallible(
                    interner,
                    args.iter().map(|t| Ok(t.lower(env)?)),
                )?;

                for (param, arg) in k
                    .binders
                    .binders
                    .iter(interner)
                    .zip(substitution.iter(interner))
                {
                    if param.kind() != arg.kind() {
                        Err(RustIrError::IncorrectParameterKind {
                            identifier: name.clone(),
                            expected: param.kind(),
                            actual: arg.kind(),
                        })?;
                    }
                }
                Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                    name: apply_name,
                    substitution,
                })
                .intern(interner))
            }

            Ty::Projection { ref proj } => Ok(chalk_ir::TyData::Alias(
                chalk_ir::AliasTy::Projection(proj.lower(env)?),
            )
            .intern(interner)),

            Ty::ForAll {
                lifetime_names,
                types,
                sig,
            } => {
                let quantified_env = env.introduce(lifetime_names.iter().map(|id| {
                    chalk_ir::WithKind::new(chalk_ir::VariableKind::Lifetime, id.str.clone())
                }))?;

                let mut lowered_tys = Vec::with_capacity(types.len());
                for ty in types {
                    lowered_tys.push(ty.lower(&quantified_env)?.cast(interner));
                }

                let function = chalk_ir::FnPointer {
                    num_binders: lifetime_names.len(),
                    substitution: Substitution::from_iter(interner, lowered_tys),
                    sig: lower_fn_sig(sig)?,
                };
                Ok(chalk_ir::TyData::Function(function).intern(interner))
            }
            Ty::Tuple { ref types } => Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                name: chalk_ir::TypeName::Tuple(types.len()),
                substitution: chalk_ir::Substitution::from_fallible(
                    interner,
                    types.iter().map(|t| Ok(t.lower(env)?)),
                )?,
            })
            .intern(interner)),

            Ty::Scalar { ty } => Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                name: chalk_ir::TypeName::Scalar(lower_scalar_type(ty)),
                substitution: chalk_ir::Substitution::empty(interner),
            })
            .intern(interner)),

            Ty::Array { ty, len } => Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                name: chalk_ir::TypeName::Array,
                substitution: chalk_ir::Substitution::from_iter(
                    interner,
                    &[
                        ty.lower(env)?.cast(interner),
                        len.lower(env)?.cast(interner),
                    ],
                ),
            })
            .intern(interner)),

            Ty::Slice { ty } => Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                name: chalk_ir::TypeName::Slice,
                substitution: chalk_ir::Substitution::from_fallible(
                    interner,
                    std::iter::once(ty.lower(env)),
                )?,
            })
            .intern(interner)),

            Ty::Raw { mutability, ty } => Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                name: chalk_ir::TypeName::Raw(lower_mutability(mutability)),
                substitution: chalk_ir::Substitution::from_fallible(
                    interner,
                    std::iter::once(Ok(ty.lower(env)?)),
                )?,
            })
            .intern(interner)),

            Ty::Ref {
                mutability,
                lifetime,
                ty,
            } => Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                name: chalk_ir::TypeName::Ref(lower_mutability(mutability)),
                substitution: chalk_ir::Substitution::from_iter(
                    interner,
                    &[
                        lifetime.lower(env)?.cast(interner),
                        ty.lower(env)?.cast(interner),
                    ],
                ),
            })
            .intern(interner)),

            Ty::Str => Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                name: chalk_ir::TypeName::Str,
                substitution: chalk_ir::Substitution::empty(interner),
            })
            .intern(interner)),

            Ty::Never => Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                name: chalk_ir::TypeName::Never,
                substitution: chalk_ir::Substitution::empty(interner),
            })
            .intern(interner)),
        }
    }
}

impl Lower for Const {
    type Lowered = chalk_ir::Const<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let interner = env.interner();
        match self {
            Const::Id(name) => {
                let parameter = env.lookup_generic_arg(name)?;
                parameter
                    .constant(interner)
                    .ok_or_else(|| RustIrError::IncorrectParameterKind {
                        identifier: name.clone(),
                        expected: Kind::Const,
                        actual: parameter.kind(),
                    })
                    .map(|c| c.clone())
            }
            Const::Value(value) => Ok(chalk_ir::ConstData {
                ty: get_type_of_u32(),
                value: chalk_ir::ConstValue::Concrete(chalk_ir::ConcreteConst { interned: *value }),
            }
            .intern(interner)),
        }
    }
}

impl Lower for GenericArg {
    type Lowered = chalk_ir::GenericArg<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let interner = env.interner();
        match self {
            GenericArg::Ty(ref t) => Ok(t.lower(env)?.cast(interner)),
            GenericArg::Lifetime(ref l) => Ok(l.lower(env)?.cast(interner)),
            GenericArg::Id(name) => env.lookup_generic_arg(&name),
            GenericArg::Const(c) => Ok(c.lower(env)?.cast(interner)),
        }
    }
}

impl Lower for Lifetime {
    type Lowered = chalk_ir::Lifetime<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let interner = env.interner();
        match self {
            Lifetime::Id { name } => {
                let parameter = env.lookup_generic_arg(&name)?;
                parameter.lifetime(interner).copied().ok_or_else(|| {
                    RustIrError::IncorrectParameterKind {
                        identifier: name.clone(),
                        expected: Kind::Lifetime,
                        actual: parameter.kind(),
                    }
                })
            }
        }
    }
}

#[instrument(level = "debug", skip(impl_, empty_env, associated_ty_value_ids))]
fn lower_impl(
    impl_: &Impl,
    empty_env: &Env,
    impl_id: ImplId<ChalkIr>,
    associated_ty_value_ids: &AssociatedTyValueIds,
) -> LowerResult<rust_ir::ImplDatum<ChalkIr>> {
    let polarity = lower_polarity(&impl_.polarity);
    let binders = empty_env.in_binders(impl_.all_parameters(), |env| {
        let trait_ref = impl_.trait_ref.lower(env)?;
        debug!(?trait_ref);

        if !polarity.is_positive() && !impl_.assoc_ty_values.is_empty() {
            Err(RustIrError::NegativeImplAssociatedValues(
                impl_.trait_ref.trait_name.clone(),
            ))?;
        }

        let where_clauses = impl_.where_clauses.lower(&env)?;
        debug!(where_clauses = ?trait_ref);
        Ok(rust_ir::ImplDatumBound {
            trait_ref,
            where_clauses,
        })
    })?;

    // lookup the ids for each of the "associated type values"
    // within the impl, which should have already assigned and
    // stored in the map
    let associated_ty_value_ids = impl_
        .assoc_ty_values
        .iter()
        .map(|atv| associated_ty_value_ids[&(impl_id, atv.name.str.clone())])
        .collect();

    debug!(?associated_ty_value_ids);

    Ok(rust_ir::ImplDatum {
        polarity,
        binders,
        impl_type: lower_impl_type(&impl_.impl_type),
        associated_ty_value_ids,
    })
}

impl Lower for Clause {
    type Lowered = Vec<chalk_ir::ProgramClause<ChalkIr>>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
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
                    constraints: chalk_ir::Constraints::empty(interner),
                    priority: ClausePriority::High,
                })
                .collect::<Vec<_>>();
            Ok(implications)
        })?;

        let clauses = implications
            .into_iter()
            .map(
                |implication: chalk_ir::Binders<chalk_ir::ProgramClauseImplication<ChalkIr>>| {
                    chalk_ir::ProgramClauseData(implication).intern(interner)
                },
            )
            .collect();
        Ok(clauses)
    }
}

fn lower_trait(
    trait_defn: &TraitDefn,
    trait_id: chalk_ir::TraitId<ChalkIr>,
    env: &Env,
) -> LowerResult<rust_ir::TraitDatum<ChalkIr>> {
    let all_parameters = trait_defn.all_parameters();
    let all_parameters_len = all_parameters.len();
    let binders = env.in_binders(all_parameters, |env| {
        if trait_defn.flags.auto {
            if all_parameters_len > 1 {
                Err(RustIrError::AutoTraitParameters(trait_defn.name.clone()))?;
            }
            if !trait_defn.where_clauses.is_empty() {
                Err(RustIrError::AutoTraitWhereClauses(trait_defn.name.clone()))?;
            }
        }

        Ok(rust_ir::TraitDatumBound {
            where_clauses: trait_defn.where_clauses.lower(env)?,
        })
    })?;

    let associated_ty_ids: Vec<_> = trait_defn
        .assoc_ty_defns
        .iter()
        .map(|defn| env.associated_ty_lookups[&(trait_id, defn.name.str.clone())].id)
        .collect();

    let trait_datum = rust_ir::TraitDatum {
        id: trait_id,
        binders,
        flags: lower_trait_flags(&trait_defn.flags),
        associated_ty_ids,
        well_known: trait_defn.well_known.map(lower_well_known_trait),
    };

    debug!(?trait_datum);

    Ok(trait_datum)
}

pub fn lower_goal(goal: &Goal, program: &LoweredProgram) -> LowerResult<chalk_ir::Goal<ChalkIr>> {
    let interner = &ChalkIr;
    let associated_ty_lookups: BTreeMap<_, _> = program
        .associated_ty_data
        .iter()
        .map(|(&associated_ty_id, datum)| {
            let trait_datum = &program.trait_data[&datum.trait_id];
            let num_trait_params = trait_datum.binders.len(interner);
            let num_addl_params = datum.binders.len(interner) - num_trait_params;
            let addl_variable_kinds =
                datum.binders.binders.as_slice(interner)[..num_addl_params].to_owned();
            let lookup = AssociatedTyLookup {
                id: associated_ty_id,
                addl_variable_kinds,
            };
            ((datum.trait_id, datum.name.clone()), lookup)
        })
        .collect();

    let auto_traits = program
        .trait_data
        .iter()
        .map(|(&trait_id, datum)| (trait_id, datum.flags.auto))
        .collect();

    let fn_def_abis: BTreeMap<_, _> = program
        .fn_def_data
        .iter()
        .map(|fn_def_data| (*fn_def_data.0, fn_def_data.1.sig.abi))
        .collect();

    let env = Env {
        adt_ids: &program.adt_ids,
        fn_def_ids: &program.fn_def_ids,
        closure_ids: &program.closure_ids,
        trait_ids: &program.trait_ids,
        opaque_ty_ids: &program.opaque_ty_ids,
        adt_kinds: &program.adt_kinds,
        fn_def_kinds: &program.fn_def_kinds,
        fn_def_abis: &fn_def_abis,
        closure_kinds: &program.closure_kinds,
        trait_kinds: &program.trait_kinds,
        opaque_ty_kinds: &program.opaque_ty_kinds,
        associated_ty_lookups: &associated_ty_lookups,
        foreign_ty_ids: &program.foreign_ty_ids,
        parameter_map: BTreeMap::new(),
        auto_traits: &auto_traits,
    };

    goal.lower(&env)
}

impl Lower for Goal {
    type Lowered = chalk_ir::Goal<ChalkIr>;

    fn lower(&self, env: &Env) -> LowerResult<Self::Lowered> {
        let interner = env.interner();
        match self {
            Goal::ForAll(ids, g) => lower_quantified(g, env, chalk_ir::QuantifierKind::ForAll, ids),
            Goal::Exists(ids, g) => lower_quantified(g, env, chalk_ir::QuantifierKind::Exists, ids),
            Goal::Implies(hyp, g) => {
                // We "elaborate" implied bounds by lowering goals like `T: Trait` and
                // `T: Trait<Assoc = U>` to `FromEnv(T: Trait)` and `FromEnv(T: Trait<Assoc = U>)`
                // in the assumptions of an `if` goal, e.g. `if (T: Trait) { ... }` lowers to
                // `if (FromEnv(T: Trait)) { ... /* this part is untouched */ ... }`.
                let where_clauses = hyp
                    .iter()
                    .flat_map(|clause| match clause.lower(env) {
                        Ok(v) => v.into_iter().map(Ok).collect(),
                        Err(e) => vec![Err(e)],
                    })
                    .map(|result| result.map(|h| h.into_from_env_clause(interner)));
                let where_clauses =
                    chalk_ir::ProgramClauses::from_fallible(interner, where_clauses);
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

fn lower_quantified(
    goal: &Goal,
    env: &Env,
    quantifier_kind: chalk_ir::QuantifierKind,
    variable_kinds: &[VariableKind],
) -> LowerResult<chalk_ir::Goal<ChalkIr>> {
    let interner = env.interner();
    if variable_kinds.is_empty() {
        return goal.lower(env);
    }

    let variable_kinds = variable_kinds.iter().map(lower_variable_kind);
    let subgoal = env.in_binders(variable_kinds, |env| goal.lower(env))?;
    Ok(chalk_ir::GoalData::Quantified(quantifier_kind, subgoal).intern(interner))
}

fn lower_well_known_trait(well_known_trait: WellKnownTrait) -> rust_ir::WellKnownTrait {
    match well_known_trait {
        WellKnownTrait::Sized => rust_ir::WellKnownTrait::Sized,
        WellKnownTrait::Copy => rust_ir::WellKnownTrait::Copy,
        WellKnownTrait::Clone => rust_ir::WellKnownTrait::Clone,
        WellKnownTrait::Drop => rust_ir::WellKnownTrait::Drop,
        WellKnownTrait::FnOnce => rust_ir::WellKnownTrait::FnOnce,
        WellKnownTrait::FnMut => rust_ir::WellKnownTrait::FnMut,
        WellKnownTrait::Fn => rust_ir::WellKnownTrait::Fn,
        WellKnownTrait::Unsize => rust_ir::WellKnownTrait::Unsize,
    }
}

trait Kinded {
    fn kind(&self) -> Kind;
}

impl Kinded for chalk_ir::VariableKind<ChalkIr> {
    fn kind(&self) -> Kind {
        match self {
            chalk_ir::VariableKind::Ty(_) => Kind::Ty,
            chalk_ir::VariableKind::Lifetime => Kind::Lifetime,
            chalk_ir::VariableKind::Const(_) => Kind::Const,
        }
    }
}

impl Kinded for chalk_ir::GenericArg<ChalkIr> {
    fn kind(&self) -> Kind {
        let interner = &ChalkIr;
        match self.data(interner) {
            chalk_ir::GenericArgData::Ty(_) => Kind::Ty,
            chalk_ir::GenericArgData::Lifetime(_) => Kind::Lifetime,
            chalk_ir::GenericArgData::Const(_) => Kind::Const,
        }
    }
}

fn lower_scalar_type(scalar_type: &ScalarType) -> chalk_ir::Scalar {
    match scalar_type {
        ScalarType::Int(int) => chalk_ir::Scalar::Int(match int {
            IntTy::I8 => chalk_ir::IntTy::I8,
            IntTy::I16 => chalk_ir::IntTy::I16,
            IntTy::I32 => chalk_ir::IntTy::I32,
            IntTy::I64 => chalk_ir::IntTy::I64,
            IntTy::I128 => chalk_ir::IntTy::I128,
            IntTy::Isize => chalk_ir::IntTy::Isize,
        }),
        ScalarType::Uint(uint) => chalk_ir::Scalar::Uint(match uint {
            UintTy::U8 => chalk_ir::UintTy::U8,
            UintTy::U16 => chalk_ir::UintTy::U16,
            UintTy::U32 => chalk_ir::UintTy::U32,
            UintTy::U64 => chalk_ir::UintTy::U64,
            UintTy::U128 => chalk_ir::UintTy::U128,
            UintTy::Usize => chalk_ir::UintTy::Usize,
        }),
        ScalarType::Float(float) => chalk_ir::Scalar::Float(match float {
            FloatTy::F32 => chalk_ir::FloatTy::F32,
            FloatTy::F64 => chalk_ir::FloatTy::F64,
        }),
        ScalarType::Bool => chalk_ir::Scalar::Bool,
        ScalarType::Char => chalk_ir::Scalar::Char,
    }
}

fn lower_mutability(mutability: &Mutability) -> chalk_ir::Mutability {
    match mutability {
        Mutability::Mut => chalk_ir::Mutability::Mut,
        Mutability::Not => chalk_ir::Mutability::Not,
    }
}

fn lower_safety(safety: &Safety) -> chalk_ir::Safety {
    match safety {
        Safety::Safe => chalk_ir::Safety::Safe,
        Safety::Unsafe => chalk_ir::Safety::Unsafe,
    }
}
