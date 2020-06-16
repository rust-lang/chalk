use crate::interner::{ChalkFnAbi, ChalkIr};
use chalk_ir::cast::{Cast, Caster};
use chalk_ir::interner::{HasInterner, Interner};
use chalk_ir::{
    self, AdtId, AssocTypeId, BoundVar, ClausePriority, DebruijnIndex, FnDefId, ImplId, OpaqueTyId,
    QuantifiedWhereClauses, Substitution, ToGenericArg, TraitId, TyKind,
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
type TraitIds = BTreeMap<Ident, chalk_ir::TraitId<ChalkIr>>;
type OpaqueTyIds = BTreeMap<Ident, chalk_ir::OpaqueTyId<ChalkIr>>;
type AdtKinds = BTreeMap<chalk_ir::AdtId<ChalkIr>, TypeKind>;
type FnDefKinds = BTreeMap<chalk_ir::FnDefId<ChalkIr>, TypeKind>;
type FnDefAbis = BTreeMap<FnDefId<ChalkIr>, <ChalkIr as Interner>::FnAbi>;
type TraitKinds = BTreeMap<chalk_ir::TraitId<ChalkIr>, TypeKind>;
type OpaqueTyKinds = BTreeMap<chalk_ir::OpaqueTyId<ChalkIr>, TypeKind>;
type AssociatedTyLookups = BTreeMap<(chalk_ir::TraitId<ChalkIr>, Ident), AssociatedTyLookup>;
type AssociatedTyValueIds =
    BTreeMap<(chalk_ir::ImplId<ChalkIr>, Ident), AssociatedTyValueId<ChalkIr>>;

type ParameterMap = BTreeMap<Ident, chalk_ir::WithKind<ChalkIr, BoundVar>>;

pub type LowerResult<T> = Result<T, RustIrError>;

#[derive(Clone, Debug)]
struct Env<'k> {
    adt_ids: &'k AdtIds,
    adt_kinds: &'k AdtKinds,
    fn_def_ids: &'k FnDefIds,
    fn_def_kinds: &'k FnDefKinds,
    fn_def_abis: &'k FnDefAbis,
    trait_ids: &'k TraitIds,
    trait_kinds: &'k TraitKinds,
    opaque_ty_ids: &'k OpaqueTyIds,
    opaque_ty_kinds: &'k OpaqueTyKinds,
    associated_ty_lookups: &'k AssociatedTyLookups,
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

enum ApplyTypeLookup {
    Adt(AdtId<ChalkIr>),
    FnDef(FnDefId<ChalkIr>),
    Opaque(OpaqueTyId<ChalkIr>),
}

const SELF: &str = "Self";
const FIXME_SELF: &str = "__FIXME_SELF__";

impl<'k> Env<'k> {
    fn lookup_generic_arg(&self, name: &Identifier) -> LowerResult<chalk_ir::GenericArg<ChalkIr>> {
        let interner = self.interner();

        if let Some(p) = self.parameter_map.get(&name.str) {
            let b = p.skip_kind();
            return match &p.kind {
                chalk_ir::VariableKind::Ty(_) => Ok(chalk_ir::TyData::BoundVar(*b)
                    .intern(interner)
                    .cast(interner)),
                chalk_ir::VariableKind::Lifetime => Ok(chalk_ir::LifetimeData::BoundVar(*b)
                    .intern(interner)
                    .cast(interner)),
                chalk_ir::VariableKind::Const(ty) => {
                    Ok(b.to_const(interner, ty.clone()).cast(interner))
                }
            };
        }

        if let Some(id) = self.adt_ids.get(&name.str) {
            let k = self.adt_kind(*id);
            if k.binders.len(interner) > 0 {
                return Err(RustIrError::IncorrectNumberOfTypeParameters {
                    identifier: name.clone(),
                    expected: k.binders.len(interner),
                    actual: 0,
                });
            } else {
                return Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                    name: chalk_ir::TypeName::Adt(*id),
                    substitution: chalk_ir::Substitution::empty(interner),
                })
                .intern(interner)
                .cast(interner));
            };
        }

        if let Some(id) = self.fn_def_ids.get(&name.str) {
            let k = self.fn_def_kind(*id);
            if k.binders.len(interner) > 0 {
                return Err(RustIrError::IncorrectNumberOfTypeParameters {
                    identifier: name.clone(),
                    expected: k.binders.len(interner),
                    actual: 0,
                });
            } else {
                return Ok(chalk_ir::TyData::Function(chalk_ir::Fn {
                    num_binders: k.binders.len(interner),
                    substitution: chalk_ir::Substitution::empty(interner),
                })
                .intern(interner)
                .cast(interner));
            }
        }

        if let Some(id) = self.opaque_ty_ids.get(&name.str) {
            return Ok(
                chalk_ir::TyData::Alias(chalk_ir::AliasTy::Opaque(chalk_ir::OpaqueTy {
                    opaque_ty_id: *id,
                    substitution: chalk_ir::Substitution::empty(interner),
                }))
                .intern(interner)
                .cast(interner),
            );
        }
        if let Some(_) = self.trait_ids.get(&name.str) {
            return Err(RustIrError::NotStruct(name.clone()));
        }

        Err(RustIrError::InvalidParameterName(name.clone()))
    }

    fn lookup_apply_type(&self, name: &Identifier) -> LowerResult<ApplyTypeLookup> {
        if let Some(_) = self.parameter_map.get(&name.str) {
            return Err(RustIrError::CannotApplyTypeParameter(name.clone()));
        }

        if let Some(id) = self.adt_ids.get(&name.str) {
            return Ok(ApplyTypeLookup::Adt(*id));
        }

        if let Some(id) = self.fn_def_ids.get(&name.str) {
            return Ok(ApplyTypeLookup::FnDef(*id));
        }

        if let Some(id) = self.opaque_ty_ids.get(&name.str) {
            return Ok(ApplyTypeLookup::Opaque(*id));
        }

        Err(RustIrError::NotStruct(name.clone()))
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
        let interner = &ChalkIr;
        let binders: Vec<_> = binders.into_iter().collect();
        let env = self.introduce(binders.iter().cloned())?;
        Ok(chalk_ir::Binders::new(
            chalk_ir::VariableKinds::from(interner, binders.iter().map(|v| v.kind.clone())),
            op(&env)?,
        ))
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
                        Err(RustIrError::AutoTraitAssociatedTypes(d.name.clone()))?;
                    }
                    for defn in &d.assoc_ty_defns {
                        let addl_variable_kinds = defn.all_parameters();
                        let lookup = AssociatedTyLookup {
                            id: AssocTypeId(next_item_id()),
                            addl_variable_kinds: addl_variable_kinds.anonymize(),
                        };
                        associated_ty_lookups
                            .insert((TraitId(raw_id), defn.name.str.clone()), lookup);
                    }
                }

                Item::Impl(d) => {
                    for atv in &d.assoc_ty_values {
                        let atv_id = AssociatedTyValueId(next_item_id());
                        associated_ty_value_ids
                            .insert((ImplId(raw_id), atv.name.str.clone()), atv_id);
                    }
                }

                _ => {}
            }
        }

        let mut adt_ids = BTreeMap::new();
        let mut fn_def_ids = BTreeMap::new();
        let mut trait_ids = BTreeMap::new();
        let mut opaque_ty_ids = BTreeMap::new();
        let mut adt_kinds = BTreeMap::new();
        let mut fn_def_kinds = BTreeMap::new();
        let mut fn_def_abis = BTreeMap::new();
        let mut trait_kinds = BTreeMap::new();
        let mut opaque_ty_kinds = BTreeMap::new();
        let mut object_safe_traits = HashSet::new();
        for (item, &raw_id) in self.items.iter().zip(&raw_ids) {
            match item {
                Item::StructDefn(defn) => {
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
                    fn_def_abis.insert(id, defn.abi.lower()?);
                }
                Item::TraitDefn(defn) => {
                    let type_kind = defn.lower_type_kind()?;
                    let id = TraitId(raw_id);
                    trait_ids.insert(type_kind.name.clone(), id);
                    trait_kinds.insert(id, type_kind);

                    if defn.flags.object_safe {
                        object_safe_traits.insert(id);
                    }
                }
                Item::OpaqueTyDefn(defn) => {
                    let type_kind = defn.lower_type_kind()?;
                    let id = OpaqueTyId(raw_id);
                    opaque_ty_ids.insert(defn.identifier.str.clone(), id);
                    opaque_ty_kinds.insert(id, type_kind);
                }
                Item::Impl(_) => continue,
                Item::Clause(_) => continue,
            };
        }

        let mut adt_data = BTreeMap::new();
        let mut fn_def_data = BTreeMap::new();
        let mut trait_data = BTreeMap::new();
        let mut well_known_traits = BTreeMap::new();
        let mut impl_data = BTreeMap::new();
        let mut associated_ty_data = BTreeMap::new();
        let mut associated_ty_values = BTreeMap::new();
        let mut opaque_ty_data = BTreeMap::new();
        let mut hidden_opaque_types = BTreeMap::new();
        let mut custom_clauses = Vec::new();
        for (item, &raw_id) in self.items.iter().zip(&raw_ids) {
            let empty_env = Env {
                adt_ids: &adt_ids,
                adt_kinds: &adt_kinds,
                fn_def_ids: &fn_def_ids,
                fn_def_kinds: &fn_def_kinds,
                fn_def_abis: &fn_def_abis,
                trait_ids: &trait_ids,
                trait_kinds: &trait_kinds,
                opaque_ty_ids: &opaque_ty_ids,
                opaque_ty_kinds: &opaque_ty_kinds,
                associated_ty_lookups: &associated_ty_lookups,
                parameter_map: BTreeMap::new(),
            };

            match *item {
                Item::StructDefn(ref d) => {
                    let adt_id = AdtId(raw_id);
                    adt_data.insert(adt_id, Arc::new(d.lower_adt(adt_id, &empty_env)?));
                }
                Item::FnDefn(ref defn) => {
                    let fn_def_id = FnDefId(raw_id);
                    fn_def_data.insert(
                        fn_def_id,
                        Arc::new(defn.lower_fn_def(fn_def_id, &empty_env)?),
                    );
                }
                Item::TraitDefn(ref trait_defn) => {
                    let trait_id = TraitId(raw_id);
                    let trait_datum = trait_defn.lower_trait(trait_id, &empty_env)?;

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
                    custom_clauses.extend(clause.lower_clause(&empty_env)?);
                }
                Item::OpaqueTyDefn(ref opaque_ty) => {
                    if let Some(&opaque_ty_id) = opaque_ty_ids.get(&opaque_ty.identifier.str) {
                        let variable_kinds = opaque_ty
                            .variable_kinds
                            .iter()
                            .map(|k| k.lower())
                            .collect::<Vec<_>>();

                        // Introduce the parameters declared on the opaque type definition.
                        // So if we have `type Foo<P1..Pn> = impl Trait<T1..Tn>`, this would introduce `P1..Pn`
                        let binders = empty_env.in_binders(variable_kinds, |env| {
                            let hidden_ty = opaque_ty.ty.lower(&env)?;
                            hidden_opaque_types.insert(opaque_ty_id, Arc::new(hidden_ty));

                            // Introduce a variable to represent the hidden "self type". This will be used in the bounds.
                            // So the `impl Trait<T1..Tn>` will be lowered to `exists<Self> { Self: Trait<T1..Tn> }`.
                            let bounds: chalk_ir::Binders<Vec<chalk_ir::Binders<_>>> = env
                                .in_binders(
                                    Some(chalk_ir::WithKind::new(
                                        chalk_ir::VariableKind::Ty(TyKind::General),
                                        Atom::from(FIXME_SELF),
                                    )),
                                    |env1| {
                                        let interner = env1.interner();
                                        Ok(opaque_ty
                                            .bounds
                                            .lower(&env1)?
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

                            Ok(OpaqueTyDatumBound { bounds })
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
            }
        }

        let program = LoweredProgram {
            adt_ids,
            fn_def_ids,
            trait_ids,
            adt_kinds,
            fn_def_kinds,
            trait_kinds,
            adt_data,
            fn_def_data,
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
        };

        Ok(program)
    }
}

trait LowerTypeKind {
    fn lower_type_kind(&self) -> LowerResult<TypeKind>;
}

trait LowerParameterMap {
    fn synthetic_parameters(&self) -> Option<chalk_ir::WithKind<ChalkIr, Ident>>;
    fn declared_parameters(&self) -> &[VariableKind];
    fn all_parameters(&self) -> Vec<chalk_ir::WithKind<ChalkIr, Ident>> {
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

    fn parameter_refs(&self) -> Vec<chalk_ir::GenericArg<ChalkIr>> {
        self.all_parameters()
            .anonymize()
            .iter()
            .zip(0..)
            .map(|p| p.to_generic_arg(self.interner()))
            .collect()
    }

    fn parameter_map(&self) -> ParameterMap {
        self.all_parameters()
            .into_iter()
            .zip((0..).map(|i| BoundVar::new(DebruijnIndex::INNERMOST, i)))
            .map(|(k, v)| {
                let (kind, name) = k.into();
                (name, chalk_ir::WithKind::new(kind, v))
            })
            .collect()
    }

    fn interner(&self) -> &ChalkIr {
        &ChalkIr
    }
}

impl LowerParameterMap for StructDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::WithKind<ChalkIr, Ident>> {
        None
    }

    fn declared_parameters(&self) -> &[VariableKind] {
        &self.variable_kinds
    }
}

impl LowerParameterMap for FnDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::WithKind<ChalkIr, Ident>> {
        None
    }

    fn declared_parameters(&self) -> &[VariableKind] {
        &self.variable_kinds
    }
}

impl LowerParameterMap for Impl {
    fn synthetic_parameters(&self) -> Option<chalk_ir::WithKind<ChalkIr, Ident>> {
        None
    }

    fn declared_parameters(&self) -> &[VariableKind] {
        &self.variable_kinds
    }
}

impl LowerParameterMap for AssocTyDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::WithKind<ChalkIr, Ident>> {
        None
    }

    fn declared_parameters(&self) -> &[VariableKind] {
        &self.variable_kinds
    }
}

impl LowerParameterMap for AssocTyValue {
    fn synthetic_parameters(&self) -> Option<chalk_ir::WithKind<ChalkIr, Ident>> {
        None
    }

    fn declared_parameters(&self) -> &[VariableKind] {
        &self.variable_kinds
    }
}

impl LowerParameterMap for TraitDefn {
    fn synthetic_parameters(&self) -> Option<chalk_ir::WithKind<ChalkIr, Ident>> {
        Some(chalk_ir::WithKind::new(
            chalk_ir::VariableKind::Ty(TyKind::General),
            Atom::from(SELF),
        ))
    }

    fn declared_parameters(&self) -> &[VariableKind] {
        &self.variable_kinds
    }
}

impl LowerParameterMap for Clause {
    fn synthetic_parameters(&self) -> Option<chalk_ir::WithKind<ChalkIr, Ident>> {
        None
    }

    fn declared_parameters(&self) -> &[VariableKind] {
        &self.variable_kinds
    }
}

fn get_type_of_u32() -> chalk_ir::Ty<ChalkIr> {
    chalk_ir::ApplicationTy {
        name: chalk_ir::TypeName::Scalar(chalk_ir::Scalar::Uint(chalk_ir::UintTy::U32)),
        substitution: Substitution::empty(&ChalkIr),
    }
    .cast(&ChalkIr)
    .intern(&ChalkIr)
}

trait LowerVariableKind {
    fn lower(&self) -> chalk_ir::WithKind<ChalkIr, Ident>;
}

impl LowerVariableKind for VariableKind {
    fn lower(&self) -> chalk_ir::WithKind<ChalkIr, Ident> {
        match self {
            VariableKind::Ty(n) => chalk_ir::WithKind::new(
                chalk_ir::VariableKind::Ty(chalk_ir::TyKind::General),
                n.str.clone(),
            ),
            VariableKind::IntegerTy(n) => chalk_ir::WithKind::new(
                chalk_ir::VariableKind::Ty(chalk_ir::TyKind::Integer),
                n.str.clone(),
            ),
            VariableKind::FloatTy(n) => chalk_ir::WithKind::new(
                chalk_ir::VariableKind::Ty(chalk_ir::TyKind::Float),
                n.str.clone(),
            ),
            VariableKind::Lifetime(n) => {
                chalk_ir::WithKind::new(chalk_ir::VariableKind::Lifetime, n.str.clone())
            }
            VariableKind::Const(ref n) => chalk_ir::WithKind::new(
                chalk_ir::VariableKind::Const(get_type_of_u32()),
                n.str.clone(),
            ),
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
        let interner = &ChalkIr;
        Ok(TypeKind {
            sort: TypeSort::Struct,
            name: self.name.str.clone(),
            binders: chalk_ir::Binders::new(
                chalk_ir::VariableKinds::from(interner, self.all_parameters().anonymize()),
                crate::Unit,
            ),
        })
    }
}

impl LowerTypeKind for FnDefn {
    fn lower_type_kind(&self) -> LowerResult<TypeKind> {
        let interner = &ChalkIr;
        Ok(TypeKind {
            sort: TypeSort::FnDef,
            name: self.name.str.clone(),
            binders: chalk_ir::Binders::new(
                chalk_ir::VariableKinds::from(interner, self.all_parameters().anonymize()),
                crate::Unit,
            ),
        })
    }
}

impl LowerWhereClauses for FnDefn {
    fn where_clauses(&self) -> &[QuantifiedWhereClause] {
        &self.where_clauses
    }
}

impl LowerWhereClauses for StructDefn {
    fn where_clauses(&self) -> &[QuantifiedWhereClause] {
        &self.where_clauses
    }
}

impl LowerTypeKind for TraitDefn {
    fn lower_type_kind(&self) -> LowerResult<TypeKind> {
        let interner = &ChalkIr;
        let binders: Vec<_> = self.variable_kinds.iter().map(|p| p.lower()).collect();
        Ok(TypeKind {
            sort: TypeSort::Trait,
            name: self.name.str.clone(),
            binders: chalk_ir::Binders::new(
                // for the purposes of the *type*, ignore `Self`:
                chalk_ir::VariableKinds::from(interner, binders.anonymize()),
                crate::Unit,
            ),
        })
    }
}

impl LowerTypeKind for OpaqueTyDefn {
    fn lower_type_kind(&self) -> LowerResult<TypeKind> {
        let interner = &ChalkIr;
        let binders: Vec<_> = self.variable_kinds.iter().map(|p| p.lower()).collect();
        Ok(TypeKind {
            sort: TypeSort::Opaque,
            name: self.identifier.str.clone(),
            binders: chalk_ir::Binders::new(
                chalk_ir::VariableKinds::from(interner, binders.anonymize()),
                crate::Unit,
            ),
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
    /// `Implemented(T: Foo)` and `ProjectionEq(<T as Foo>::Item = U)`.
    fn lower(&self, env: &Env) -> LowerResult<Vec<T>>;
}

impl LowerWhereClause<chalk_ir::WhereClause<ChalkIr>> for WhereClause {
    fn lower(&self, env: &Env) -> LowerResult<Vec<chalk_ir::WhereClause<ChalkIr>>> {
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
        };
        Ok(where_clauses)
    }
}
impl LowerWhereClause<chalk_ir::QuantifiedWhereClause<ChalkIr>> for QuantifiedWhereClause {
    fn lower(&self, env: &Env) -> LowerResult<Vec<chalk_ir::QuantifiedWhereClause<ChalkIr>>> {
        let variable_kinds = self.variable_kinds.iter().map(|pk| pk.lower());
        let binders = env.in_binders(variable_kinds, |env| Ok(self.where_clause.lower(env)?))?;
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
            DomainGoal::Compatible => vec![chalk_ir::DomainGoal::Compatible(())],
            DomainGoal::DownstreamType { ty } => {
                vec![chalk_ir::DomainGoal::DownstreamType(ty.lower(env)?)]
            }
            DomainGoal::Reveal => vec![chalk_ir::DomainGoal::Reveal(())],
            DomainGoal::ObjectSafe { id } => {
                vec![chalk_ir::DomainGoal::ObjectSafe(env.lookup_trait(id)?)]
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
            LeafGoal::UnifyGenericArgs { a, b } => chalk_ir::EqGoal {
                a: a.lower(env)?.cast(interner),
                b: b.lower(env)?.cast(interner),
            }
            .cast::<chalk_ir::Goal<ChalkIr>>(interner),
        })
    }
}

trait LowerAdtDefn {
    fn lower_adt(
        &self,
        adt_id: chalk_ir::AdtId<ChalkIr>,
        env: &Env,
    ) -> LowerResult<rust_ir::AdtDatum<ChalkIr>>;
}

impl LowerAdtDefn for StructDefn {
    fn lower_adt(
        &self,
        adt_id: chalk_ir::AdtId<ChalkIr>,
        env: &Env,
    ) -> LowerResult<rust_ir::AdtDatum<ChalkIr>> {
        if self.flags.fundamental && self.all_parameters().len() != 1 {
            Err(RustIrError::InvalidFundamentalTypesParameters(
                self.name.clone(),
            ))?;
        }

        let binders = env.in_binders(self.all_parameters(), |env| {
            let fields: LowerResult<_> = self.fields.iter().map(|f| f.ty.lower(env)).collect();
            let where_clauses = self.lower_where_clauses(env)?;

            Ok(rust_ir::AdtDatumBound {
                fields: fields?,
                where_clauses,
            })
        })?;

        let flags = rust_ir::AdtFlags {
            upstream: self.flags.upstream,
            fundamental: self.flags.fundamental,
            phantom_data: self.flags.phantom_data,
        };

        Ok(rust_ir::AdtDatum {
            id: adt_id,
            binders,
            flags,
        })
    }
}

trait LowerFnDefn {
    fn lower_fn_def(
        &self,
        fn_def_id: chalk_ir::FnDefId<ChalkIr>,
        env: &Env,
    ) -> LowerResult<rust_ir::FnDefDatum<ChalkIr>>;
}

impl LowerFnDefn for FnDefn {
    fn lower_fn_def(
        &self,
        fn_def_id: chalk_ir::FnDefId<ChalkIr>,
        env: &Env,
    ) -> LowerResult<rust_ir::FnDefDatum<ChalkIr>> {
        let binders = env.in_binders(self.all_parameters(), |env| {
            let args: LowerResult<_> = self.argument_types.iter().map(|t| t.lower(env)).collect();
            let where_clauses = self.lower_where_clauses(env)?;
            let return_type = self.return_type.lower(env)?;

            let inputs_and_output = env.in_binders(vec![], |_| {
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
            abi: self.abi.lower()?,
            binders,
        })
    }
}

trait LowerFnAbi {
    fn lower(&self) -> LowerResult<ChalkFnAbi>;
}

impl LowerFnAbi for FnAbi {
    fn lower(&self) -> LowerResult<ChalkFnAbi> {
        match self.0.as_ref() {
            "Rust" => Ok(ChalkFnAbi::Rust),
            "C" => Ok(ChalkFnAbi::C),
            _ => Err(RustIrError::InvalidExternAbi(self.0.clone())),
        }
    }
}

trait LowerTraitRef {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::TraitRef<ChalkIr>>;
}

impl LowerTraitRef for TraitRef {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::TraitRef<ChalkIr>> {
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

trait LowerTraitBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::TraitBound<ChalkIr>>;
}

impl LowerTraitBound for TraitBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::TraitBound<ChalkIr>> {
        let interner = &ChalkIr;
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

trait LowerAliasEqBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::AliasEqBound<ChalkIr>>;
}

impl LowerAliasEqBound for AliasEqBound {
    fn lower(&self, env: &Env) -> LowerResult<rust_ir::AliasEqBound<ChalkIr>> {
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
        let variable_kinds = self.variable_kinds.iter().map(|pk| pk.lower());
        let binders = env.in_binders(variable_kinds, |env| Ok(self.bound.lower(env)?))?;
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

trait LowerProjectionTy {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::ProjectionTy<ChalkIr>>;
}

impl LowerProjectionTy for ProjectionTy {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::ProjectionTy<ChalkIr>> {
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
        let lookup = match env
            .associated_ty_lookups
            .get(&(trait_id.into(), name.str.clone()))
        {
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
            substitution: chalk_ir::Substitution::from(interner, args),
        })
    }
}

trait LowerTy {
    /// Lower from the AST to Chalk's Rust IR
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Ty<ChalkIr>>;
}

impl LowerTy for Ty {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Ty<ChalkIr>> {
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
                        Ok(QuantifiedWhereClauses::from(
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
                    ApplyTypeLookup::Adt(id) => (chalk_ir::TypeName::Adt(id), env.adt_kind(id)),
                    ApplyTypeLookup::FnDef(id) => {
                        (chalk_ir::TypeName::FnDef(id), env.fn_def_kind(id))
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
            } => {
                let quantified_env = env.introduce(lifetime_names.iter().map(|id| {
                    chalk_ir::WithKind::new(chalk_ir::VariableKind::Lifetime, id.str.clone())
                }))?;

                let mut lowered_tys = Vec::with_capacity(types.len());
                for ty in types {
                    lowered_tys.push(ty.lower(&quantified_env)?.cast(interner));
                }

                let function = chalk_ir::Fn {
                    num_binders: lifetime_names.len(),
                    substitution: Substitution::from(interner, lowered_tys),
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
                name: chalk_ir::TypeName::Scalar(ast_scalar_to_chalk_scalar(ty.clone())),
                substitution: chalk_ir::Substitution::empty(interner),
            })
            .intern(interner)),

            Ty::Array { ty, len } => Ok(chalk_ir::TyData::Apply(chalk_ir::ApplicationTy {
                name: chalk_ir::TypeName::Array,
                substitution: chalk_ir::Substitution::from(
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
                name: chalk_ir::TypeName::Raw(ast_mutability_to_chalk_mutability(
                    mutability.clone(),
                )),
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
                name: chalk_ir::TypeName::Ref(ast_mutability_to_chalk_mutability(
                    mutability.clone(),
                )),
                substitution: chalk_ir::Substitution::from(
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

trait LowerConst {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Const<ChalkIr>>;
}

impl LowerConst for Const {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Const<ChalkIr>> {
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
                value: chalk_ir::ConstValue::Concrete(chalk_ir::ConcreteConst {
                    interned: value.clone(),
                }),
            }
            .intern(interner)),
        }
    }
}

trait LowerGenericArg {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::GenericArg<ChalkIr>>;
}

impl LowerGenericArg for GenericArg {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::GenericArg<ChalkIr>> {
        let interner = env.interner();
        match self {
            GenericArg::Ty(ref t) => Ok(t.lower(env)?.cast(interner)),
            GenericArg::Lifetime(ref l) => Ok(l.lower(env)?.cast(interner)),
            GenericArg::Id(name) => env.lookup_generic_arg(&name),
            GenericArg::Const(c) => Ok(c.lower(env)?.cast(interner)),
        }
    }
}

trait LowerLifetime {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Lifetime<ChalkIr>>;
}

impl LowerLifetime for Lifetime {
    fn lower(&self, env: &Env) -> LowerResult<chalk_ir::Lifetime<ChalkIr>> {
        let interner = env.interner();
        match self {
            Lifetime::Id { name } => {
                let parameter = env.lookup_generic_arg(&name)?;
                parameter
                    .lifetime(interner)
                    .map(|l| l.clone())
                    .ok_or_else(|| RustIrError::IncorrectParameterKind {
                        identifier: name.clone(),
                        expected: Kind::Lifetime,
                        actual: parameter.kind(),
                    })
            }
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
    #[instrument(level = "debug", skip(self, empty_env, associated_ty_value_ids))]
    fn lower_impl(
        &self,
        empty_env: &Env,
        impl_id: ImplId<ChalkIr>,
        associated_ty_value_ids: &AssociatedTyValueIds,
    ) -> LowerResult<rust_ir::ImplDatum<ChalkIr>> {
        let polarity = self.polarity.lower();
        let binders = empty_env.in_binders(self.all_parameters(), |env| {
            let trait_ref = self.trait_ref.lower(env)?;
            debug!("trait_ref = {:?}", trait_ref);

            if !polarity.is_positive() && !self.assoc_ty_values.is_empty() {
                Err(RustIrError::NegativeImplAssociatedValues(
                    self.trait_ref.trait_name.clone(),
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
            .map(|atv| associated_ty_value_ids[&(impl_id, atv.name.str.clone())])
            .collect();

        debug!("associated_ty_value_ids = {:?}", associated_ty_value_ids);

        Ok(rust_ir::ImplDatum {
            polarity,
            binders,
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
                    Err(RustIrError::AutoTraitParameters(self.name.clone()))?;
                }
                if !self.where_clauses.is_empty() {
                    Err(RustIrError::AutoTraitWhereClauses(self.name.clone()))?;
                }
            }

            Ok(rust_ir::TraitDatumBound {
                where_clauses: self.lower_where_clauses(env)?,
            })
        })?;

        let associated_ty_ids: Vec<_> = self
            .assoc_ty_defns
            .iter()
            .map(|defn| env.associated_ty_lookups[&(trait_id, defn.name.str.clone())].id)
            .collect();

        let trait_datum = rust_ir::TraitDatum {
            id: trait_id,
            binders: binders,
            flags: self.flags.lower(),
            associated_ty_ids,
            well_known: self.well_known.map(|t| t.lower()),
        };

        debug!("trait_datum={:?}", trait_datum);

        Ok(trait_datum)
    }
}

pub trait LowerGoal<A> {
    fn lower(&self, arg: &A) -> LowerResult<chalk_ir::Goal<ChalkIr>>;
}

impl LowerGoal<LoweredProgram> for Goal {
    fn lower(&self, program: &LoweredProgram) -> LowerResult<chalk_ir::Goal<ChalkIr>> {
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
        let fn_def_abis: BTreeMap<_, _> = program
            .fn_def_data
            .iter()
            .map(|fn_def_data| (*fn_def_data.0, fn_def_data.1.abi))
            .collect();

        let env = Env {
            adt_ids: &program.adt_ids,
            fn_def_ids: &program.fn_def_ids,
            trait_ids: &program.trait_ids,
            opaque_ty_ids: &program.opaque_ty_ids,
            adt_kinds: &program.adt_kinds,
            fn_def_kinds: &program.fn_def_kinds,
            fn_def_abis: &fn_def_abis,
            trait_kinds: &program.trait_kinds,
            opaque_ty_kinds: &program.opaque_ty_kinds,
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
                let where_clauses = hyp
                    .into_iter()
                    .flat_map(|h| h.lower_clause(env).apply_result())
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

trait LowerQuantifiedGoal {
    fn lower_quantified(
        &self,
        env: &Env,
        quantifier_kind: chalk_ir::QuantifierKind,
        variable_kinds: &[VariableKind],
    ) -> LowerResult<chalk_ir::Goal<ChalkIr>>;
}

impl LowerQuantifiedGoal for Goal {
    fn lower_quantified(
        &self,
        env: &Env,
        quantifier_kind: chalk_ir::QuantifierKind,
        variable_kinds: &[VariableKind],
    ) -> LowerResult<chalk_ir::Goal<ChalkIr>> {
        let interner = env.interner();
        if variable_kinds.is_empty() {
            return self.lower(env);
        }

        let variable_kinds = variable_kinds.iter().map(|pk| pk.lower());
        let subgoal = env.in_binders(variable_kinds, |env| self.lower(env))?;
        Ok(chalk_ir::GoalData::Quantified(quantifier_kind, subgoal).intern(interner))
    }
}

trait LowerWellKnownTrait {
    fn lower(&self) -> rust_ir::WellKnownTrait;
}

impl LowerWellKnownTrait for WellKnownTrait {
    fn lower(&self) -> rust_ir::WellKnownTrait {
        match self {
            Self::Sized => rust_ir::WellKnownTrait::Sized,
            Self::Copy => rust_ir::WellKnownTrait::Copy,
            Self::Clone => rust_ir::WellKnownTrait::Clone,
            Self::Drop => rust_ir::WellKnownTrait::Drop,
            Self::FnOnce => rust_ir::WellKnownTrait::FnOnce,
            Self::FnMut => rust_ir::WellKnownTrait::FnMut,
            Self::Fn => rust_ir::WellKnownTrait::Fn,
            Self::Unsize => rust_ir::WellKnownTrait::Unsize,
        }
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

impl Kinded for VariableKind {
    fn kind(&self) -> Kind {
        match *self {
            VariableKind::Ty(_) => Kind::Ty,
            VariableKind::IntegerTy(_) => Kind::Ty,
            VariableKind::FloatTy(_) => Kind::Ty,
            VariableKind::Lifetime(_) => Kind::Lifetime,
            VariableKind::Const(_) => Kind::Const,
        }
    }
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

impl Kinded for chalk_ir::GenericArgData<ChalkIr> {
    fn kind(&self) -> Kind {
        match self {
            chalk_ir::GenericArgData::Ty(_) => Kind::Ty,
            chalk_ir::GenericArgData::Lifetime(_) => Kind::Lifetime,
            chalk_ir::GenericArgData::Const(_) => Kind::Const,
        }
    }
}

impl Kinded for chalk_ir::GenericArg<ChalkIr> {
    fn kind(&self) -> Kind {
        let interner = &ChalkIr;
        self.data(interner).kind()
    }
}

fn ast_scalar_to_chalk_scalar(scalar: ScalarType) -> chalk_ir::Scalar {
    match scalar {
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

fn ast_mutability_to_chalk_mutability(mutability: Mutability) -> chalk_ir::Mutability {
    match mutability {
        Mutability::Mut => chalk_ir::Mutability::Mut,
        Mutability::Not => chalk_ir::Mutability::Not,
    }
}
