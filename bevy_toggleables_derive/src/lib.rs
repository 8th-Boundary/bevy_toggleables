use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, spanned::Spanned, Data, DeriveInput, Error, Field, Fields, FieldsNamed,
    Meta, Result, Type, TypePath,
};

#[proc_macro_derive(Toggleable, attributes(toggleable))]
pub fn derive_toggleable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match expand_toggleable(input) {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand_toggleable(input: DeriveInput) -> Result<proc_macro2::TokenStream> {
    let ident = input.ident;
    let generics = input.generics;

    let fields = match input.data {
        Data::Struct(data) => match data.fields {
            Fields::Named(FieldsNamed { named, .. }) => named.into_iter().collect::<Vec<_>>(),
            Fields::Unnamed(fields) => {
                return Err(Error::new(
                    fields.span(),
                    "Toggleable only supports named-field structs",
                ))
            }
            Fields::Unit => {
                return Err(Error::new(
                    ident.span(),
                    "Toggleable requires a named bool field",
                ))
            }
        },
        Data::Enum(data) => {
            return Err(Error::new(
                data.enum_token.span,
                "Toggleable only supports structs",
            ))
        }
        Data::Union(data) => {
            return Err(Error::new(
                data.union_token.span,
                "Toggleable only supports structs",
            ))
        }
    };

    let mut errors: Option<Error> = None;
    let mut marked: Option<usize> = None;
    let mut active_named: Option<usize> = None;
    let mut active_bool: Option<usize> = None;

    for (index, field) in fields.iter().enumerate() {
        if has_toggleable_attr(field)? && marked.replace(index).is_some() {
            combine_error(
                &mut errors,
                Error::new_spanned(field, "multiple #[toggleable] fields are not allowed"),
            );
        }

        if field
            .ident
            .as_ref()
            .is_some_and(|field_ident| field_ident == "active")
        {
            active_named = Some(index);
            if is_bool_type(&field.ty) {
                active_bool = Some(index);
            }
        }
    }

    let selected_index = if let Some(marked_index) = marked {
        if !is_bool_type(&fields[marked_index].ty) {
            combine_error(
                &mut errors,
                Error::new_spanned(
                    &fields[marked_index],
                    "#[toggleable] field must have type bool",
                ),
            );
        }

        if let Some(active_index) = active_bool {
            if active_index != marked_index {
                combine_error(
                    &mut errors,
                    Error::new_spanned(
                        &fields[active_index],
                        "an explicit #[toggleable] field conflicts with a separate active: bool field",
                    ),
                );
            }
        }

        marked_index
    } else if let Some(active_index) = active_bool {
        active_index
    } else if let Some(active_index) = active_named {
        combine_error(
            &mut errors,
            Error::new_spanned(
                &fields[active_index],
                "the active field must have type bool or be marked with #[toggleable]",
            ),
        );
        active_index
    } else {
        combine_error(
            &mut errors,
            Error::new(
                ident.span(),
                "expected a `bool` toggle field named `active` or marked with #[toggleable]",
            ),
        );
        0
    };

    if let Some(error) = errors {
        return Err(error);
    }

    let field = &fields[selected_index];
    let field_ident = field
        .ident
        .as_ref()
        .expect("named struct fields always have identifiers");

    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics ::bevy_toggleables::ToggleableComponent for #ident #ty_generics #where_clause {
            #[inline]
            fn is_active(&self) -> bool {
                self.#field_ident
            }

            #[inline]
            fn set_active(&mut self, active: bool) {
                self.#field_ident = active;
            }
        }
    })
}

fn has_toggleable_attr(field: &Field) -> Result<bool> {
    let mut found = false;
    for attr in &field.attrs {
        if attr.path().is_ident("toggleable") {
            if !matches!(&attr.meta, Meta::Path(_)) {
                return Err(Error::new_spanned(
                    attr,
                    "#[toggleable] does not take arguments",
                ));
            }
            found = true;
        }
    }
    Ok(found)
}

fn is_bool_type(ty: &Type) -> bool {
    match ty {
        Type::Path(TypePath { qself: None, path }) => path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "bool" && segment.arguments.is_empty()),
        _ => false,
    }
}

fn combine_error(acc: &mut Option<Error>, error: Error) {
    if let Some(existing) = acc.as_mut() {
        existing.combine(error);
    } else {
        *acc = Some(error);
    }
}
