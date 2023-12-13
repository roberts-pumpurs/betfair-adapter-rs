use heck::{ToPascalCase, ToSnakeCase};
use proc_macro2::{Ident, Span, TokenStream};

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash, Default)]
pub(crate) struct Name(pub(crate) String);

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct Comment {
    item: String,
}

impl Comment {
    pub(crate) fn module_comment(&self) -> TokenStream {
        const MODULE_COMMENT: &str = "///";
        pad_with(MODULE_COMMENT, &self.item)
    }

    pub(crate) fn object_comment(&self) -> TokenStream {
        const OBJECT_COMMENT: &str = "///";
        pad_with(OBJECT_COMMENT, &self.item)
    }
}

impl Name {
    pub(crate) fn ident_snake(&self) -> Ident {
        Ident::new(&self.0.to_snake_case(), Span::call_site())
    }

    pub(crate) fn ident_pascal(&self) -> Ident {
        Ident::new(&self.0.to_pascal_case(), Span::call_site())
    }

    pub(crate) fn module_comment(&self) -> TokenStream {
        const MODULE_COMMENT: &str = "///";
        pad_with(MODULE_COMMENT, &self.0)
    }
}
fn pad_with(pad: &str, text: impl AsRef<str>) -> proc_macro2::TokenStream {
    let text = text
        .as_ref()
        .split_terminator('\n')
        .collect::<Vec<_>>()
        .join(format!("\n{}", pad).as_str());
    quote::quote! {
        #[doc = #text]
    }
}

impl Comment {
    pub(crate) fn new(item: String) -> Self {
        Self { item }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub(crate) struct DataTypeParameter(String);

impl DataTypeParameter {
    pub(crate) fn new(s: String) -> Self {
        Self(s)
    }

    pub(crate) fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for DataTypeParameter {
    fn from(s: String) -> Self {
        Self::new(s)
    }
}
