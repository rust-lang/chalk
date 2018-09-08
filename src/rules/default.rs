use ir::*;
use rust_ir::*;
use solve::infer::InferenceTable;
use ir::cast::Cast;

impl Program {
    pub fn add_default_impls(&mut self) {
        // For each auto trait `MyAutoTrait` and for each struct/type `MyStruct`
        for auto_trait in self.trait_data
            .values()
            .filter(|t| t.binders.value.flags.auto)
        {
            for struct_datum in self.struct_data.values() {
                // `MyStruct: MyAutoTrait`
                let trait_ref = TraitRef {
                    trait_id: auto_trait.binders.value.trait_ref.trait_id,
                    parameters: vec![
                        ParameterKind::Ty(Ty::Apply(struct_datum.binders.value.self_ty.clone())),
                    ],
                };

                // If a positive or negative impl is already provided for a type family
                // which includes `MyStruct`, we do not generate a default impl.
                if self.impl_provided_for(trait_ref.clone(), struct_datum) {
                    continue;
                }

                self.default_impl_data.push(DefaultImplDatum {
                    binders: Binders {
                        binders: struct_datum.binders.binders.clone(),
                        value: DefaultImplDatumBound {
                            trait_ref,
                            accessible_tys: struct_datum.binders.value.fields.clone(),
                        },
                    },
                });
            }
        }
    }

    fn impl_provided_for(&self, trait_ref: TraitRef, struct_datum: &StructDatum) -> bool {
        let goal: DomainGoal = trait_ref.cast();

        let mut infer = InferenceTable::new();

        let goal = infer.instantiate_binders_existentially(&(&struct_datum.binders.binders, &goal));

        for impl_datum in self.impl_data.values() {
            // We retrieve the trait ref given by the positive impl (even if the actual impl is negative)
            let impl_goal: DomainGoal = impl_datum
                .binders
                .value
                .trait_ref
                .trait_ref()
                .clone()
                .cast();

            let impl_goal =
                infer.instantiate_binders_existentially(&(&impl_datum.binders.binders, &impl_goal));

            // We check whether the impl `MyStruct: (!)MyAutoTrait` unifies with an existing impl.
            // Examples:
            //
            // ```
            // struct MyStruct;
            // impl<T> Send for T where T: Foo { }
            // ```
            // `MyStruct: Send` unifies with `T: Send` so no default impl is generated for `MyStruct`.
            //
            // ```
            // struct MyStruct;
            // impl<T> Send for Vec<T> where T: Foo { }
            // ```
            // `Vec<i32>: Send` unifies with `Vec<T>: Send` so no default impl is generated for `Vec<i32>`.
            // But a default impl is generated for `MyStruct`.
            //
            // ```
            // struct MyStruct;
            // impl<T> !Send for T where T: Foo { }
            // ```
            // `MyStruct: !Send` unifies with `T: !Send` so no default impl is generated for `MyStruct`.
            if infer.unify(&Environment::new(), &goal, &impl_goal).is_ok() {
                return true;
            }
        }

        false
    }
}
