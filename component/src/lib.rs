use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{FnArg, Result};

struct Component(syn::ItemFn);

impl Parse for Component {
    fn parse(input: ParseStream) -> Result<Self> {
        let func = input.parse::<syn::ItemFn>()?;

        if let Some(async_token) = func.sig.asyncness {
            let message = "Component cannot be async";

            return Err(syn::Error::new(async_token.span, message));
        }

        let name = &func.sig.ident;
        let first_char = name.to_string().chars().next().unwrap();

        if first_char.is_ascii_lowercase() {
            let message = "Expected first char at upper case";

            return Err(syn::Error::new(name.span(), message));
        }

        Ok(Self(func))
    }
}

impl ToTokens for Component {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let func = &self.0;

        let args = func.sig.inputs.iter();
        let prop_names = args.clone().map(|a| {
            let FnArg::Typed(a) = a else { unreachable!() };
            let syn::Pat::Ident(pt) = a.pat.as_ref() else {
                unreachable!()
            };
            pt.ident.clone()
        });
        let props = args.map(Prop::from).collect::<Vec<_>>();

        let body = func.block.as_ref();
        let name = &func.sig.ident;
        let vis = &func.vis;
        let mod_name = format!("__{}_private", name.to_string().to_ascii_lowercase());
        let mod_name = syn::Ident::new(mod_name.as_str(), name.span());
        let generics = &func.sig.generics;

        quote!(
            #vis use #mod_name::#name;

            impl #generics ::cercis::html::component::Component for #name #generics {
                fn render(&self) -> String {
                    {
                        let Self { #(#prop_names,)* } = self;
                        #body
                    }.render()
                }
            }

            mod #mod_name {
                use ::cercis::system::*;
                use ::cercis::prelude::*;

                #[derive(typed_builder::TypedBuilder)]
                #[builder(doc, crate_module_path=typed_builder)]
                pub struct #name #generics {#(#props,)*}
            }
        )
        .to_tokens(tokens)
    }
}

struct Prop {
    prop: FnArg,
    is_opt: bool,
}

impl From<&FnArg> for Prop {
    fn from(value: &FnArg) -> Self {
        let mut is_opt = false;
        let value = value.clone();

        if let FnArg::Typed(pt) = &value {
            is_opt = pt.ty.to_token_stream().to_string().contains("Option <");
        }

        Self {
            prop: value,
            is_opt,
        }
    }
}

impl ToTokens for Prop {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let attr = quote!(#[builder(default, setter(strip_option))]);
        let prop = &self.prop;

        if let FnArg::Typed(pt) = prop {
            if pt.ty.to_token_stream().to_string().as_str() == "Element" {
                quote!(#[builder(default = Element::default())] pub #prop).to_tokens(tokens);
                return;
            }
        }

        match self.is_opt {
            true => quote!(#attr pub #prop),
            false => quote!(pub #prop),
        }
        .to_tokens(tokens)
    }
}

/// Macro ```#[component]``` write component for ```rsx!``` like default Rust function
///
/// > All components must start with a capital letter
///
/// # Examples
///
/// ## Declaration
///
/// ```
/// use cercis::prelude::*;
///
/// #[component]
/// fn MyComponent() -> Element {
///   rsx!(h1 { "My component!" })
/// }
/// ```
///
/// ## Props
///
/// ```
/// use cercis::prelude::*;
///
/// #[component]
/// fn MyComponent<'a>(text: &'a str) -> Element {
///   rsx!(div {
///     h1 { "My component!" }
///     p { "{text}" }
///   })
/// }
/// ```
///
/// ## Optional props
///
/// ```
/// use cercis::prelude::*;
///
/// #[component]
/// fn MyComponent<'a>(text: Option<&'a str>) -> Element {
///   let text = text.unwrap_or("empty");
///
///   rsx!(div {
///     h1 { "My component!" }
///     p { "{text}" }
///   })
/// }
/// ```
#[proc_macro_attribute]
pub fn component(_: TokenStream, input: TokenStream) -> TokenStream {
    match syn::parse::<Component>(input.clone()) {
        Ok(component) => {
            let body = component.into_token_stream();
            body.into()
        }
        Err(err) => err.to_compile_error().into(),
    }
}
