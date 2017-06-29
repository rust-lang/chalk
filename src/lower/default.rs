use ir::*;
use solve::infer::InferenceTable;
use cast::Cast;

impl Program {
    pub(super) fn add_default_impls(&mut self) {
        // For each auto trait `MyAutoTrait` and for each struct/type `MyStruct`
        for auto_trait in self.trait_data.values().filter(|t| t.binders.value.auto) {
            for struct_datum in self.struct_data.values() {

                // `MyStruct: MyAutoTrait`
                let trait_ref = TraitRef {
                    trait_id: auto_trait.binders.value.trait_ref.trait_id,
                    parameters: vec![
                        ParameterKind::Ty(Ty::Apply(struct_datum.binders.value.self_ty.clone()))
                    ]
                };

                // If a positive or negative impl is already provided for a type family
                // which includes `MyStruct`, we do not generate a default impl.
                if self.provide_impl_for(PolarizedTraitRef::Positive(trait_ref.clone()), struct_datum) ||
                   self.provide_impl_for(PolarizedTraitRef::Negative(trait_ref.clone()), struct_datum) {
                    continue;
                }

                self.default_impl_data.push(DefaultImplDatum {
                    binders: Binders {
                        binders: struct_datum.binders.binders.clone(),
                        value: DefaultImplDatumBound {
                            trait_ref,
                            accessible_tys: struct_datum.binders.value.fields.clone(),
                        }
                    }
                });
            }
        }
    }

    fn provide_impl_for(&self, trait_ref: PolarizedTraitRef, struct_datum: &StructDatum) -> bool {
        let goal: DomainGoal = trait_ref.cast();

        let env = Environment::new();
        let mut infer = InferenceTable::new();

        let goal = infer.instantiate_in(env.universe, struct_datum.binders.binders.clone(), &goal);

        for impl_datum in self.impl_data.values() {
            let impl_goal: DomainGoal = impl_datum.binders.value.trait_ref.clone().cast();
            let impl_goal = infer.instantiate_in(env.universe, impl_datum.binders.binders.clone(), &impl_goal);

            // We check whether the impl `MyStruct: (!)MyAutoTrait` unifies with an existing impl.
            // Examples:
            //
            // ```
            // struct MyStruct;
            // impl<T> Send for T where T: Foo { }
            // ```
            // No default impl is generated for `MyStruct`.
            //
            // ```
            // struct MyStruct;
            // impl<T> Send for Vec<T> where T: Foo { }
            // ```
            // No default impl is generated for `Vec<i32>`, but a default impl is generated for `MyStruct`.
            //
            // ```
            // struct MyStruct;
            // impl<T> !Send for T where T: Foo { }
            // ```
            // No default impl is generated for `MyStruct`.
            if infer.unify(&Environment::new(), &goal, &impl_goal).is_ok() {
                return true;
            }
        }

        false
    }
}
