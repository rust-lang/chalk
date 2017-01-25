use cast::Cast;
use chalk_rust_parse::ast::*;
use lalrpop_intern::intern;
use errors::*;
use ir;
use std::collections::HashMap;

mod test;

type TypeIds = HashMap<ir::Identifier, ir::ItemId>;
type TypeKinds = HashMap<ir::ItemId, ir::TypeKind>;
type AssociatedTyInfos = HashMap<(ir::ItemId, ir::Identifier), AssociatedTyInfo>;
type ParameterMap = HashMap<ir::ParameterKind<ir::Identifier>, usize>;

#[derive(Debug)]
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
        if let Some(k) = self.parameter_map.get(&ir::ParameterKind::Lifetime(name.str)) {
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
        where I: IntoIterator<Item = ir::ParameterKind<ir::Identifier>>,
              I::IntoIter: ExactSizeIterator,
    {
        let binders = binders.into_iter().enumerate().map(|(i, k)| (k, i));
        let len = binders.len();
        let parameter_map: ParameterMap =
            self.parameter_map.iter()
                              .map(|(&k, &v)| (k, v + len))
                              .chain(binders)
                              .collect();
        Env { parameter_map, ..*self }
    }

    fn in_binders<I, T, OP>(&self, binders: I, op: OP) -> Result<ir::Binders<T>>
        where I: IntoIterator<Item = ir::ParameterKind<ir::Identifier>>,
              I::IntoIter: ExactSizeIterator,
              OP: FnOnce(&Self) -> Result<T>,
    {
        let binders: Vec<_> = binders.into_iter().collect();
        let env = self.introduce(binders.iter().cloned());
        Ok(ir::Binders { binders: binders.anonymize(), value: op(&env)? })
    }
}

pub trait LowerProgram {
    fn lower(&self) -> Result<ir::Program>;
}

impl LowerProgram for Program {
    fn lower(&self) -> Result<ir::Program> {
        let mut flat_items = vec![];
        flatten(intern("crate"), &self.items, &mut flat_items);

        fn flatten<'a>(crate_name: ir::Identifier,
                       items: &'a [Item],
                       into: &mut Vec<(ir::Identifier, &'a Item)>) {
            for item in items {
                match *item {
                    Item::StructDefn(_) |
                    Item::TraitDefn(_) |
                    Item::Impl(_) =>
                        into.push((crate_name, item)),
                    Item::CrateDefn(ref defn) =>
                        flatten(defn.name.str, &defn.items, into),
                }
            }
        }

        let mut index = 0;
        let mut next_item_id = || -> ir::ItemId {
            let i = index;
            index += 1;
            ir::ItemId { index: i }
        };

        // Make a vector mapping each thing in `flat_items` to an id,
        // based just on its position:
        let item_ids: Vec<_> =
            flat_items
            .iter()
            .map(|_| next_item_id())
            .collect();

        // Create ids for associated types
        let mut associated_ty_infos = HashMap::new();
        for (&(_, item), &item_id) in flat_items.iter().zip(&item_ids) {
            if let Item::TraitDefn(ref d) = *item {
                for defn in &d.assoc_ty_defns {
                    let addl_parameter_kinds = defn.all_parameters();
                    let info = AssociatedTyInfo { id: next_item_id(), addl_parameter_kinds };
                    associated_ty_infos.insert((item_id, defn.name.str), info);
                }
            }
        }

        let mut type_ids = HashMap::new();
        let mut type_kinds = HashMap::new();
        for (&(crate_name, item), &item_id) in flat_items.iter().zip(&item_ids) {
            let k = match *item {
                Item::StructDefn(ref d) => d.lower_type_kind(crate_name)?,
                Item::TraitDefn(ref d) => d.lower_type_kind(crate_name)?,
                Item::Impl(_) => continue,
                Item::CrateDefn(_) => continue,
            };
            type_ids.insert(k.name, item_id);
            type_kinds.insert(item_id, k);
        }

        let mut trait_data = HashMap::new();
        let mut impl_data = HashMap::new();
        let mut associated_ty_data = HashMap::new();
        for (&(crate_name, item), &item_id) in flat_items.iter().zip(&item_ids) {
            let parameter_map = item.parameter_map();
            let env = Env {
                type_ids: &type_ids,
                type_kinds: &type_kinds,
                associated_ty_infos: &associated_ty_infos,
                parameter_map: parameter_map,
            };
            match *item {
                Item::CrateDefn(_) => { }
                Item::StructDefn(ref _d) => {
                    // where_clauses.insert(item_id, d.lower_where_clauses(&env)?);
                }
                Item::TraitDefn(ref d) => {
                    trait_data.insert(item_id, d.lower_trait(crate_name, &env)?);

                    let trait_data = &trait_data[&item_id];
                    for defn in &d.assoc_ty_defns {
                        let info = &associated_ty_infos[&(item_id, defn.name.str)];

                        // Given `trait Foo<'a, T>`, produce a trait ref like
                        //
                        //     <Ty::Var(0): Foo<Lifetime::Var(1), Ty::Var(2)>
                        //
                        // Note that within a set of binders (i.e.,
                        // the things declared on the impl), we don't
                        // use "deBruijn" indexing but rather just
                        // straight-up indexing.
                        //
                        // This will be the where-clause for the
                        // associated type. (IOW, to project this
                        // associated type, one must prove that the
                        // trait applies.)
                        let offset = info.addl_parameter_kinds.len();
                        let trait_ref = ir::TraitRef {
                            trait_id: item_id,
                            parameters: {
                                trait_data.parameter_kinds
                                          .anonymize()
                                          .iter()
                                          .zip(offset..)
                                          .map(|p| p.to_parameter())
                                          .collect()
                            },
                        };

                        let mut parameter_kinds = defn.all_parameters();
                        parameter_kinds.extend(trait_data.parameter_kinds.iter().cloned());

                        associated_ty_data.insert(info.id, ir::AssociatedTyData {
                            trait_id: item_id,
                            name: defn.name.str,
                            parameter_kinds: parameter_kinds,
                            where_clauses: vec![ir::WhereClause::Implemented(trait_ref)]
                        });
                    }
                }
                Item::Impl(ref d) => {
                    impl_data.insert(item_id, d.lower_impl(crate_name, &env)?);
                }
            }
        }

        // Construct the set of *clauses*; these are sort of a compiled form
        // of the data above that always has the form:
        //
        //       forall P0...Pn. Something :- Conditions
        let mut program_clauses = vec![];

        for impl_datum in impl_data.values() {
            program_clauses.push(impl_datum.to_program_clause());

            for atv in &impl_datum.assoc_ty_values {
                program_clauses.push(atv.to_program_clause(impl_datum));
            }
        }

        Ok(ir::Program { type_ids, type_kinds, trait_data, impl_data, associated_ty_data, program_clauses })
    }
}

trait LowerTypeKind {
    fn lower_type_kind(&self, crate_name: ir::Identifier) -> Result<ir::TypeKind>;
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
            Item::CrateDefn(..) |
            Item::Impl(..) => None,
        }
    }

    fn declared_parameters(&self) -> &[ParameterKind] {
        match *self {
            Item::TraitDefn(ref d) => d.declared_parameters(),
            Item::StructDefn(ref d) => &d.parameter_kinds,
            Item::Impl(ref d) => &d.parameter_kinds,
            Item::CrateDefn(_) => &[],
        }
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
    fn lower_type_kind(&self, crate_name: ir::Identifier) -> Result<ir::TypeKind> {
        Ok(ir::TypeKind {
            sort: ir::TypeSort::Struct,
            name: self.name.str,
            crate_name: crate_name,
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
    fn lower_type_kind(&self, crate_name: ir::Identifier) -> Result<ir::TypeKind> {
        Ok(ir::TypeKind {
            sort: ir::TypeSort::Trait,
            name: self.name.str,
            crate_name: crate_name,

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

trait LowerWhereClause<T> {
    fn lower(&self, env: &Env) -> Result<T>;
}

/// Lowers a where-clause in the context of a clause; this is limited
/// to the kinds of where-clauses users can actually type in Rust.
impl LowerWhereClause<ir::WhereClause> for WhereClause {
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
            WhereClause::NotImplemented { .. } => {
                bail!("negative trait refs cannot be where-clauses")
            }
        })
    }
}

/// Lowers a where-clause in the context of a goal; this is richer in
/// terms of the legal sorts of where-clauses that can appear, because
/// it includes all the sorts of things that the compiler must verify.
impl LowerWhereClause<ir::WhereClauseGoal> for WhereClause {
    fn lower(&self, env: &Env) -> Result<ir::WhereClauseGoal> {
        Ok(match *self {
            WhereClause::Implemented { .. } |
            WhereClause::ProjectionEq { .. } => {
                let wc: ir::WhereClause = self.lower(env)?;
                wc.cast()
            }
            WhereClause::NotImplemented { .. } => {
                unimplemented!() // oh the irony
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

        let parameters = self.args.iter().map(|a| Ok(a.lower(env)?)).collect::<Result<Vec<_>>>()?;

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
        let ProjectionTy { ref trait_ref, ref name, ref args } = *self;
        let ir::TraitRef { trait_id, parameters: trait_parameters } = trait_ref.lower(env)?;
        let info = match env.associated_ty_infos.get(&(trait_id, name.str)) {
            Some(info) => info,
            None => bail!("no associated type `{}` defined in trait", name.str)
        };
        let mut args: Vec<_> = try!(args.iter().map(|a| a.lower(env)).collect());

        if args.len() != info.addl_parameter_kinds.len() {
            bail!("wrong number of parameters for associated type (expected {}, got {})",
                  info.addl_parameter_kinds.len(), args.len())
        }

        args.extend(trait_parameters);

        Ok(ir::ProjectionTy { associated_ty_id: info.id, parameters: args })
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

            Ty::ForAll { ref lifetime_names, ref ty } => {
                let quantified_env =
                    env.introduce(lifetime_names
                                  .iter()
                                  .map(|id| ir::ParameterKind::Lifetime(id.str)));
                let ty = ty.lower(&quantified_env)?;
                let quantified_ty = ir::QuantifiedTy { num_binders: lifetime_names.len(), ty };
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
            Lifetime::Id { name } => {
                match env.lookup_lifetime(name)? {
                    LifetimeLookup::Parameter(d) => Ok(ir::Lifetime::Var(d))
                }
            }
        }
    }
}

trait LowerImpl {
    fn lower_impl(&self, crate_name: ir::Identifier, env: &Env) -> Result<ir::ImplData>;
}

impl LowerImpl for Impl {
    fn lower_impl(&self, crate_name: ir::Identifier, env: &Env) -> Result<ir::ImplData> {
        let trait_ref = self.trait_ref.lower(env)?;
        let trait_id = trait_ref.trait_id;
        Ok(ir::ImplData {
            crate_name: crate_name,
            trait_ref: trait_ref,
            parameter_kinds: self.parameter_kinds.iter().map(|p| p.lower()).collect(),
            assoc_ty_values: try!(self.assoc_ty_values.iter().map(|v| v.lower(trait_id, env)).collect()),
            where_clauses: self.lower_where_clauses(&env)?,
        })
    }
}

trait LowerAssocTyValue {
    fn lower(&self, trait_id: ir::ItemId, env: &Env) -> Result<ir::AssocTyValue>;
}

impl LowerAssocTyValue for AssocTyValue {
    fn lower(&self, trait_id: ir::ItemId, env: &Env) -> Result<ir::AssocTyValue> {
        let info = &env.associated_ty_infos[&(trait_id, self.name.str)];
        let value = env.in_binders(self.all_parameters(), |env| {
            Ok(ir::AssocTyValueData {
                ty: self.value.lower(env)?,
                where_clauses: self.where_clauses.lower(env)?,
            })
        })?;
        Ok(ir::AssocTyValue { associated_ty_id: info.id, value: value })
    }
}

trait LowerTrait {
    fn lower_trait(&self, crate_name: ir::Identifier, env: &Env) -> Result<ir::TraitData>;
}

impl LowerTrait for TraitDefn {
    fn lower_trait(&self, crate_name: ir::Identifier, env: &Env) -> Result<ir::TraitData> {
        Ok(ir::TraitData {
            crate_name: crate_name,
            parameter_kinds: self.all_parameters(),
            where_clauses: self.lower_where_clauses(&env)?,
        })
    }
}

pub trait LowerGoal<A> {
    fn lower(&self, arg: &A) -> Result<Box<ir::Goal>>;
}

impl LowerGoal<ir::Program> for Goal {
    fn lower(&self, program: &ir::Program) -> Result<Box<ir::Goal>> {
        let associated_ty_infos: HashMap<_, _> =
            program.associated_ty_data
                   .iter()
                   .map(|(&associated_ty_id, datum)| {
                       let trait_data = &program.trait_data[&datum.trait_id];
                       let num_trait_params = trait_data.parameter_kinds.len();
                       let num_addl_params = datum.parameter_kinds.len() - num_trait_params;
                       let addl_parameter_kinds = datum.parameter_kinds[..num_addl_params].to_owned();
                       let info = AssociatedTyInfo { id: associated_ty_id, addl_parameter_kinds };
                       ((datum.trait_id, datum.name), info)
                   })
                   .collect();

        let env = Env {
            type_ids: &program.type_ids,
            type_kinds: &program.type_kinds,
            associated_ty_infos: &associated_ty_infos,
            parameter_map: HashMap::new()
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
                Ok(Box::new(ir::Goal::Leaf(wc.lower(env)?))),
            Goal::WellFormed(ref ty) =>
                Ok(Box::new(ir::Goal::Leaf(ir::WhereClauseGoal::WellFormed(ty.lower(env)?)))),
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

        let parameter_kinds = parameter_kinds.iter().map(|pk| pk.lower());
        let subgoal = env.in_binders(parameter_kinds, |env| self.lower(env))?;
        Ok(Box::new(ir::Goal::Quantified(quantifier_kind, subgoal)))
    }
}

impl ir::ImplData {
    /// Given `impl<T: Clone> Clone for Vec<T>`, generate:
    ///
    /// ```notrust
    /// forall<T> { (Vec<T>: Clone) :- (T: Clone) }
    /// ```
    fn to_program_clause(&self) -> ir::ProgramClause {
        ir::ProgramClause {
            implication: ir::Binders {
                binders: self.parameter_kinds.anonymize(),
                value: ir::ProgramClauseImplication {
                    consequence: self.trait_ref.clone().cast(),
                    conditions: self.where_clauses.clone().cast(),
                },
            }
        }
    }
}

impl ir::AssocTyValue {
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
    fn to_program_clause(&self, impl_datum: &ir::ImplData) -> ir::ProgramClause {
        // Begin with the innermost parameters (`'a`) and then add those from impl (`T`).
        let all_binders: Vec<_> =
            self.value.binders
                      .iter()
                      .cloned()
                      .chain(impl_datum.parameter_kinds.anonymize())
                      .collect();

        // Assemble the full list of conditions for projection to be valid.
        // This comes in two parts, marked as (1) and (2) in example above:
        //
        // 1. require that the trait is implemented
        // 2. any where-clauses from the `type` declaration in the impl
        let conditions: Vec<ir::Goal> =
            Some(impl_datum.trait_ref.up_shift(self.value.binders.len()).cast())
            .into_iter()
            .chain(self.value.value.where_clauses.clone().cast())
            .collect();

        // The consequence is that the normalization holds.
        let consequence = {
            // First add refs to the bound parameters (`'a`, in above example)
            let parameters = self.value.binders.iter().zip(0..).map(|p| p.to_parameter());

            // Then add the trait-ref parameters (`Vec<T>`, in above example)
            let parameters = parameters.chain(impl_datum.trait_ref.parameters
                                              .iter()
                                              .map(|p| p.up_shift(self.value.binders.len())));

            // Construct normalization predicate
            ir::Normalize {
                projection: ir::ProjectionTy {
                    associated_ty_id: self.associated_ty_id,
                    parameters: parameters.collect(),
                },
                ty: self.value.value.ty.clone()
            }
        };

        ir::ProgramClause {
            implication: ir::Binders {
                binders: all_binders,
                value: ir::ProgramClauseImplication {
                    consequence: consequence.cast(),
                    conditions: conditions,
                }
            }
        }
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
            ir::ParameterKind::Lifetime(_) =>
                ir::ParameterKind::Lifetime(ir::Lifetime::Var(index)),
            ir::ParameterKind::Ty(_) =>
                ir::ParameterKind::Ty(ir::Ty::Var(index)),
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
