use crate::prelude::*;

pub enum ParseNode {
    Element(ParseElement),
    Content(String),
}

impl Parse for ParseNode {
    fn parse(input: ParseStream) -> Result<Self> {
        let name = input.parse::<syn::Ident>()?;
        let mut attributes = Vec::new();
        let mut children = Vec::new();

        let block;
        braced!(block in input);

        while block.peek(syn::Ident) && block.peek2(Token![:]) {
            attributes.push(block.parse::<ParseAttribute>()?)
        }

        while !block.is_empty() {
            if block.peek(syn::LitStr) {
                let text = block.parse::<syn::LitStr>()?.value();

                children.push(Self::Content(text));
            } else if block.peek(syn::Ident) {
                children.push(block.parse::<Self>()?)
            }
        }

        Ok(Self::Element(ParseElement {
            name: name.to_string(),
            attributes,
            children,
        }))
    }
}

impl ToTokens for ParseNode {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        match self {
            ParseNode::Element(element) => {
                let name = element.name.as_str();
                let attributes = element.attributes.as_slice();
                let children = element.children.as_slice();

                quote!(Node::Element(Element::new(#name)#(.attr(#attributes))*#(.child(#children))*)).to_tokens(tokens)
            }
            ParseNode::Content(content) => quote!(Node::content(#content)).to_tokens(tokens),
        }
    }
}
